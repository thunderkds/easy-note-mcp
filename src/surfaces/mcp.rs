//! The MCP server (FR-012): seven tools and one prompt (D37) over stdio, or streamable HTTP on
//! `127.0.0.1` only. Each tool is a thin call into the core (`store`, `query`, `context`, `guard`,
//! `ai`). `note_add` adds one new note file (and, with `detail`, one AI file); `note_enrich` writes
//! one AI file in `.easy-note/ai/`; no tool edits, deletes or moves a note. The prompt
//! `enrich_notes` is fixed text: easy-note runs no model (DDR 0004).
//!
//! The tokio runtime is built here, inside `serve`, so the CLI path never starts it (NFR-010).
//! Tool output is JSON text; a failure is a tool error whose text went through
//! `guard::mask_for_output`, because a store error can quote what the caller sent.

use std::net::Ipv4Addr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use rmcp::handler::server::tool::ToolCallContext;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolRequestParams, CallToolResponse, ContentBlock, PromptMessage, Role};
use rmcp::schemars;
use rmcp::service::RequestContext;
use rmcp::transport::stdio;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::{
    prompt, prompt_handler, prompt_router, tool, tool_handler, tool_router, ErrorData, RoleServer,
    ServerHandler, ServiceExt,
};
use serde::Deserialize;
use serde_json::{json, Map, Value as Json};

use crate::core::ai::{self, AiError, MAX_PENDING};
use crate::core::context;
use crate::core::format::{Note, Value};
use crate::core::guard;
use crate::core::query::{self, Filters, DEFAULT_LIMIT};
use crate::core::store::{Store, StoreError};

pub const DEFAULT_PORT: u16 = 8769;
const HOST: Ipv4Addr = Ipv4Addr::LOCALHOST;
/// A client that says it has roots but never answers must not hold a note back.
const ROOTS_TIMEOUT: Duration = Duration::from_secs(2);

const ADD_DESCRIPTION: &str = "Save one short note for the person in their notes folder. \
Call note_add only when the person asks to note something. \
When the person's message starts with `note:` or `/note`, call this at once with the rest of the \
message as `text`, then reply in one line. \
Pass `cwd` (the folder you work in) so the note records its repo, branch and task.";

/// The fixed text of the `enrich_notes` prompt (FR-031). The client's model does the work.
const ENRICH_PROMPT: &str =
    "Add AI detail to the person's easy-note notes. Use one subagent for the whole run; \
keep the work out of this conversation.

1. Call note_pending_enrich. It lists the notes with no AI detail or stale detail, each with \
id, text, created and, when known, repo, task and session.
2. For each note, find what the person was doing when they wrote it:
   - If the note has a session and that Claude Code session's transcript file exists \
(projects/*/<session>.jsonl in the Claude Code config dir), read only the entries from the \
10 minutes before created, at most 40 entries.
   - If there is no session but there is a repo, use a Claude Code transcript of that repo \
only if exactly one transcript has entries in those 10 minutes; read it the same way.
   - Otherwise, or if the transcript file is gone, enrich from the note text, repo, branch and \
task only (note_read gives the branch), with no context claim.
3. Write a title (at most 120 chars), a kind (bug, idea, question, todo or other), a summary \
(at most 600 chars) and at most 5 suggested tags (a-z, 0-9, _, / and -). Explain in your own \
words; never quote the transcript or the note, and never copy a secret.
4. Call note_enrich once per note with id, title, kind, summary, tags_suggested and model \
(your model name). It writes only the AI detail file; the note itself never changes.
5. Reply with one line: how many notes got detail.";

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddArgs {
    /// The note text. Secrets are masked before it is written.
    text: String,
    /// Tags, without the leading `#`.
    tags: Option<Vec<String>>,
    /// A task id such as `T120`. Default: taken from the branch name.
    task: Option<String>,
    /// The folder you work in. Default: the client's roots, then the server's folder.
    cwd: Option<String>,
    /// Optional detail you already know (at most 600 chars), saved apart from the note as AI
    /// detail with model `capture`. Never the person's words.
    detail: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PendingArgs {
    /// At most 20. Default 20.
    limit: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EnrichArgs {
    /// The note id from note_pending_enrich.
    id: String,
    /// At most 120 chars.
    title: String,
    /// One of `bug`, `idea`, `question`, `todo`, `other`.
    kind: String,
    /// At most 600 chars, in your own words.
    summary: String,
    /// At most 5 tags (a-z, 0-9, _, / and -), without the leading `#`.
    #[serde(default)]
    tags_suggested: Vec<String>,
    /// Your model name.
    model: String,
}

/// Filters shared by `note_search`, `note_list` and `note_stats`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FilterArgs {
    repo: Option<String>,
    task: Option<String>,
    tag: Option<String>,
    /// `cli`, `mcp`, `hook`, `quick` or `inbox`.
    source: Option<String>,
    /// First day, `YYYY-MM-DD`.
    since: Option<String>,
    /// Last day, `YYYY-MM-DD`.
    until: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchArgs {
    /// Words that must all appear in the title, body or tags.
    query: String,
    #[serde(flatten)]
    filters: FilterArgs,
    /// At most 50. Default 20.
    limit: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListArgs {
    #[serde(flatten)]
    filters: FilterArgs,
    /// One day, `YYYY-MM-DD`.
    day: Option<String>,
    /// At most 50. Default 20.
    limit: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReadArgs {
    /// A note id such as `20261114-164713-a1b2`.
    id: String,
}

fn filters(args: FilterArgs, day: Option<String>) -> Filters {
    Filters {
        repo: args.repo,
        task: args.task,
        tag: args.tag,
        source: args.source,
        since: args.since,
        until: args.until,
        day,
    }
}

/// The text of a tool error: masked, never the raw message.
fn fail(error: impl std::fmt::Display) -> String {
    guard::mask_for_output(&error.to_string())
}

/// The text of a store error on an id the caller sent: a bad id is quoted (and escaped) by the
/// store, so it is not shown; an unknown id is shown masked.
fn id_error(error: StoreError) -> String {
    match error {
        StoreError::BadId(_) => "not a note id".to_string(),
        StoreError::NotFound(id) => format!("no note with id {}", masked_id(&id)),
        other => fail(other),
    }
}

fn ai_error(error: AiError) -> String {
    match error {
        AiError::Store(error) => id_error(error),
        refused => fail(refused),
    }
}

/// The text of a protocol error from the router. For a wrongly typed argument serde quotes the
/// value it could not read, and it escapes it, so the guard cannot see a secret in it: only the
/// messages that quote no value are kept.
fn argument_error(message: &str) -> String {
    match message.strip_prefix("failed to deserialize parameters: ") {
        Some(detail) if !detail.starts_with("missing field") => {
            "invalid arguments: check the tool's input schema".to_string()
        }
        _ => guard::mask_for_output(message),
    }
}

/// Run blocking core work (files, git) off the async thread.
async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|_| "internal error".to_string())?
}

fn json_text(value: &impl serde::Serialize) -> Result<String, String> {
    serde_json::to_string(value).map_err(fail)
}

/// An inbox note's id is its file name, which the sync app chose and which may hold a secret.
/// Every id and path that leaves through MCP is masked, as the CLI does through `safe`.
fn masked_id(id: &str) -> String {
    guard::mask_for_output(id)
}

fn note_json(note: &Note) -> Json {
    let mut out = Map::new();
    for (key, value) in &note.fields {
        out.insert(
            key.clone(),
            match value {
                Value::Int(n) => json!(n),
                Value::Str(s) if key == "id" => json!(masked_id(s)),
                Value::Str(s) => json!(s),
                Value::List(list) => json!(list),
            },
        );
    }
    out.insert("body".into(), json!(note.body));
    Json::Object(out)
}

/// The folders the client offers as roots. Empty when it has none, does not support them,
/// errors, or is slow: the caller then falls back to the process folder.
#[allow(deprecated)]
async fn client_roots(context: &RequestContext<RoleServer>) -> Vec<String> {
    if !context
        .client_capabilities()
        .is_some_and(|caps| caps.roots.is_some())
    {
        return vec![];
    }
    match tokio::time::timeout(ROOTS_TIMEOUT, context.peer.list_roots()).await {
        Ok(Ok(found)) => found.roots.into_iter().map(|root| root.uri).collect(),
        _ => vec![],
    }
}

#[derive(Clone, Default)]
pub struct EasyNote {
    /// `None`: open the notes dir from `EASY_NOTE_DIR`, config.toml or `~/easy-notes` per call.
    store: Option<Store>,
}

impl EasyNote {
    pub fn new() -> EasyNote {
        EasyNote::default()
    }

    /// A server on a given store (tests).
    pub fn with_store(store: Store) -> EasyNote {
        EasyNote { store: Some(store) }
    }

    fn store(&self) -> Result<Store, String> {
        match &self.store {
            Some(store) => Ok(store.clone()),
            None => Store::open().map_err(fail),
        }
    }
}

#[tool_router]
impl EasyNote {
    #[tool(description = ADD_DESCRIPTION)]
    async fn note_add(
        &self,
        context: RequestContext<RoleServer>,
        Parameters(args): Parameters<AddArgs>,
    ) -> Result<String, String> {
        if args.text.trim().is_empty() {
            return Err("note text is empty".into());
        }
        let store = self.store()?;
        let client = context
            .client_info()
            .map(|info| info.name)
            .filter(|name| !name.is_empty());
        let cwd = args.cwd.filter(|cwd| !cwd.trim().is_empty());
        let detail = args.detail.filter(|d| !d.trim().is_empty());
        // Checked before anything is written; the cap is on the masked text, as stored.
        if let Some(detail) = &detail {
            ai::check_len("detail", &guard::mask(detail).0, ai::MAX_SUMMARY).map_err(fail)?;
        }
        let roots = if cwd.is_none() {
            client_roots(&context).await
        } else {
            vec![]
        };
        blocking(move || {
            let found =
                context::capture(args.task.as_deref(), cwd.as_deref().map(Path::new), &roots);
            let mut fields = found.fields();
            fields.push(("client", client.as_deref()));
            let tags = args.tags.unwrap_or_default();
            let added = store.add(&args.text, "mcp", &tags, &fields).map_err(fail)?;
            // Report what was written (masked), not what was asked for.
            let note = store.read(&added.id).map_err(fail)?;
            // The note is kept whatever happens to its AI file.
            let detail_saved =
                detail.map(|detail| ai::capture(&store, &added.id, &note.title(), &detail).is_ok());
            let mut answer = json!({
                "id": added.id,
                "path": added.path,
                "repo": note.get_str("repo"),
                "branch": note.get_str("branch"),
                "task": note.get_str("task"),
                "tags": note.fields.get("tags").map(|t| match t {
                    Value::List(list) => list.clone(),
                    _ => vec![],
                }).unwrap_or_default(),
            });
            if let Some(saved) = detail_saved {
                answer["detail_saved"] = json!(saved);
            }
            json_text(&answer)
        })
        .await
    }

    #[tool(
        description = "Search the person's notes: every word of `query` must appear in the title, body or tags. At most 50 items, each with a short snippet, plus `total` and `truncated`. Read-only.",
        annotations(read_only_hint = true)
    )]
    async fn note_search(
        &self,
        Parameters(args): Parameters<SearchArgs>,
    ) -> Result<String, String> {
        let store = self.store()?;
        blocking(move || {
            let found = filters(args.filters, None);
            let limit = args.limit.unwrap_or(DEFAULT_LIMIT);
            let mut page = query::search_in(&store, &args.query, &found, limit).map_err(fail)?;
            for item in &mut page.items {
                item.id = masked_id(&item.id);
            }
            json_text(&page)
        })
        .await
    }

    #[tool(
        description = "List the person's notes, newest first, with optional filters. At most 50 items, each with a short snippet, plus `total` and `truncated`. Read-only.",
        annotations(read_only_hint = true)
    )]
    async fn note_list(&self, Parameters(args): Parameters<ListArgs>) -> Result<String, String> {
        let store = self.store()?;
        blocking(move || {
            let found = filters(args.filters, args.day);
            let limit = args.limit.unwrap_or(DEFAULT_LIMIT);
            let mut page = query::list_notes_in(&store, &found, limit).map_err(fail)?;
            for item in &mut page.items {
                item.id = masked_id(&item.id);
            }
            json_text(&page)
        })
        .await
    }

    #[tool(
        description = "Read one full note by id. Secrets in the text are masked. Read-only.",
        annotations(read_only_hint = true)
    )]
    async fn note_read(&self, Parameters(args): Parameters<ReadArgs>) -> Result<String, String> {
        let store = self.store()?;
        blocking(move || {
            let note = store.read(&args.id).map_err(|error| match error {
                // The store quotes (and escapes) a bad id, which could be a secret.
                StoreError::BadId(_) => "not a note id".to_string(),
                other => fail(other),
            })?;
            let mut out = note_json(&note);
            if let Some((detail, stale)) = ai::read(&store, note.id()) {
                out["ai"] = json!({
                    "label": ai::LABEL,
                    "stale": stale,
                    "title": detail.title,
                    "kind": detail.kind,
                    "summary": detail.summary,
                    "tags_suggested": detail.tags_suggested,
                    "model": detail.model,
                    "at": detail.at,
                });
            }
            json_text(&out)
        })
        .await
    }

    #[tool(
        description = "Count the person's notes by repo, task, tag, source and month, and list files that could not be read (`skipped`). Optional filters. Read-only.",
        annotations(read_only_hint = true)
    )]
    async fn note_stats(&self, Parameters(args): Parameters<FilterArgs>) -> Result<String, String> {
        let store = self.store()?;
        blocking(move || {
            let mut stats = query::stats_in(&store, &filters(args, None)).map_err(fail)?;
            for skipped in &mut stats.skipped {
                skipped.path = masked_id(&skipped.path);
            }
            json_text(&stats)
        })
        .await
    }

    #[tool(
        description = "List the person's notes that have no AI detail yet, or stale detail, newest first: id, text, created, and repo, task and session when known. At most 20 (`limit`). Read-only. Used by the enrich_notes prompt.",
        annotations(read_only_hint = true)
    )]
    async fn note_pending_enrich(
        &self,
        Parameters(args): Parameters<PendingArgs>,
    ) -> Result<String, String> {
        let store = self.store()?;
        blocking(move || {
            let limit = args.limit.unwrap_or(MAX_PENDING);
            let mut page = ai::pending(&store, limit).map_err(ai_error)?;
            for item in &mut page.items {
                item.id = masked_id(&item.id);
            }
            json_text(&page)
        })
        .await
    }

    #[tool(
        description = "Save AI detail for one note, from the enrich_notes prompt: title (at most 120 chars), kind (bug, idea, question, todo or other), summary (at most 600 chars), at most 5 suggested tags, and your model name. Writes only the note's AI detail file, labelled as not the person's words, replacing older detail; the note itself never changes. Returns id and based_on."
    )]
    async fn note_enrich(
        &self,
        Parameters(args): Parameters<EnrichArgs>,
    ) -> Result<String, String> {
        let store = self.store()?;
        blocking(move || {
            let input = ai::Input {
                title: &args.title,
                kind: &args.kind,
                summary: &args.summary,
                tags_suggested: &args.tags_suggested,
                model: &args.model,
            };
            let detail = ai::enrich(&store, &args.id, &input).map_err(ai_error)?;
            json_text(&json!({"id": masked_id(&detail.id), "based_on": detail.based_on}))
        })
        .await
    }
}

#[prompt_router]
impl EasyNote {
    #[prompt(
        name = "enrich_notes",
        description = "Add AI detail (title, kind, summary, suggested tags) to the person's notes that have none, in one subagent. The notes never change."
    )]
    async fn enrich_notes(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(Role::User, ENRICH_PROMPT)]
    }
}

#[tool_handler(
    name = "easy-note",
    instructions = "The person's notes. Call note_add only when the person asks to note something. note_enrich writes only AI detail files, when the person runs the enrich_notes prompt; the other tools only read."
)]
#[prompt_handler]
impl ServerHandler for EasyNote {
    /// The router answers; every error text then passes the guard. A wrongly typed argument makes
    /// the SDK quote the value it could not read, which may be a secret.
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let call = ToolCallContext::new(self, request, context);
        match Self::tool_router().call(call).await {
            Ok(CallToolResponse::Complete(mut result)) if result.is_error == Some(true) => {
                for block in &mut result.content {
                    if let ContentBlock::Text(text) = block {
                        text.text = argument_error(&text.text);
                    }
                }
                Ok(CallToolResponse::Complete(result))
            }
            Err(mut error) => {
                error.message = argument_error(&error.message).into();
                error.data = None;
                Err(error)
            }
            other => other,
        }
    }
}

/// How `serve` talks: stdin/stdout, or HTTP on `127.0.0.1:<port>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Stdio,
    Http { port: u16 },
}

/// HTTP binds `127.0.0.1` and nothing else (NFR-001). Checked before anything binds.
pub fn check_host(host: &str) -> Result<(), String> {
    match host.parse::<Ipv4Addr>() {
        Ok(addr) if addr == HOST => Ok(()),
        _ => Err(format!(
            "--host {host:?} is refused: HTTP mode binds 127.0.0.1 only"
        )),
    }
}

/// Serve MCP until the client closes stdin (stdio) or the process is stopped (HTTP).
pub fn serve(mode: Mode) -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        match mode {
            Mode::Stdio => {
                let running = EasyNote::new().serve(stdio()).await?;
                running.waiting().await?;
                Ok(())
            }
            Mode::Http { port } => serve_http(port).await,
        }
    })
}

async fn serve_http(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind((HOST, port))
        .await
        .map_err(|e| format!("cannot bind {HOST}:{port}: {e}"))?;
    // The Host header is already limited to loopback names; a browser page must also come from one.
    let config = StreamableHttpServerConfig::default()
        .with_allowed_origins(["http://localhost:*", "http://127.0.0.1:*"]);
    let service = StreamableHttpService::new(
        || Ok(EasyNote::new()),
        Arc::new(LocalSessionManager::default()),
        config,
    );
    let router = axum::Router::new().nest_service("/mcp", service);
    eprintln!(
        "easy-note: MCP on http://{HOST}:{}/mcp",
        listener.local_addr()?.port()
    );
    axum::serve(listener, router).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN: &str = concat!("gh", "p_EXAMPLEexampleEXAMPLEexampleEXAMPLE123456");

    #[test]
    fn error_text_is_masked() {
        assert!(!fail(format!("cannot read {TOKEN}")).contains(TOKEN));
        assert!(!argument_error(&format!("tool {TOKEN} not found")).contains(TOKEN));
    }

    #[test]
    fn a_quoted_argument_value_never_reaches_the_error_text() {
        let quoted = format!(
            "failed to deserialize parameters: invalid type: string \"password=\\\"x{TOKEN}\\\"\", expected a sequence"
        );
        let shown = argument_error(&quoted);
        assert!(
            !shown.contains("password") && !shown.contains(TOKEN),
            "{shown}"
        );
        let missing = "failed to deserialize parameters: missing field `text`";
        assert_eq!(argument_error(missing), missing);
    }

    #[test]
    fn only_127_0_0_1_passes_the_host_check() {
        assert!(check_host("127.0.0.1").is_ok());
        for host in [
            "0.0.0.0",
            "localhost",
            "::",
            "::1",
            "127.0.0.2",
            "10.0.0.1",
            "",
        ] {
            assert!(check_host(host).is_err(), "{host:?} passed");
        }
    }
}
