//! easy-note's own dashboard window (FR-042, DDR 0009): a tao window with a wry (WebKitGTK)
//! webview, behind the `window` cargo feature. The page is embedded in the binary and served
//! only through the `easy-note://` scheme; it reaches the notes only through IPC answered by the
//! pure `api::handle`. The CSP, the asset table, the URL check and the reply script are plain
//! functions here so the default build tests them; only the shell needs the feature.

pub mod api;
pub mod single;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use serde_json::Value;

use crate::core::context::Context;
use crate::core::format::check_text;
use crate::core::guard;
use crate::core::store::Store;

/// The Content-Security-Policy of every page response: nothing but the app's own scheme, and no
/// inline style or script (the same string as the page's own `<meta>` policy).
pub const CSP: &str = "default-src easy-note:";
/// The page's origin: the only one that may be navigated to or send IPC.
pub const ORIGIN: &str = "easy-note://localhost";
/// The page's address.
pub const START: &str = "easy-note://localhost/";
/// The quick window's page (HUN-289): the capture dialog alone.
pub const QUICK_START: &str = "easy-note://localhost/quick.html";
/// What the shell runs when the window gets focus: the page asks `board` again.
pub const REFRESH: &str = "window.__easyNote && window.__easyNote.refresh()";

const HTML: &str = "text/html; charset=utf-8";
const JS: &str = "text/javascript; charset=utf-8";
const CSS: &str = "text/css; charset=utf-8";
const WOFF2: &str = "font/woff2";

/// The embedded file for a request path, with its MIME type; `None` for any other path.
pub fn asset(path: &str) -> Option<(&'static str, &'static [u8])> {
    match path {
        "/" | "/index.html" => Some((HTML, include_bytes!("assets/index.html"))),
        "/app.js" => Some((JS, include_bytes!("assets/app.js"))),
        "/app.css" => Some((CSS, include_bytes!("assets/app.css"))),
        "/ipc.js" => Some((JS, include_bytes!("assets/ipc.js"))),
        "/capture.js" => Some((JS, include_bytes!("assets/capture.js"))),
        "/settings.js" => Some((JS, include_bytes!("assets/settings.js"))),
        "/quick.html" => Some((HTML, include_bytes!("assets/quick.html"))),
        "/quick.js" => Some((JS, include_bytes!("assets/quick.js"))),
        "/quick.css" => Some((CSS, include_bytes!("assets/quick.css"))),
        "/fonts/ibm-plex-sans-latin-400-normal.woff2" => Some((
            WOFF2,
            include_bytes!("assets/fonts/ibm-plex-sans-latin-400-normal.woff2"),
        )),
        "/fonts/ibm-plex-sans-latin-500-normal.woff2" => Some((
            WOFF2,
            include_bytes!("assets/fonts/ibm-plex-sans-latin-500-normal.woff2"),
        )),
        "/fonts/ibm-plex-sans-latin-600-normal.woff2" => Some((
            WOFF2,
            include_bytes!("assets/fonts/ibm-plex-sans-latin-600-normal.woff2"),
        )),
        "/fonts/ibm-plex-mono-latin-400-normal.woff2" => Some((
            WOFF2,
            include_bytes!("assets/fonts/ibm-plex-mono-latin-400-normal.woff2"),
        )),
        "/fonts/ibm-plex-mono-latin-500-normal.woff2" => Some((
            WOFF2,
            include_bytes!("assets/fonts/ibm-plex-mono-latin-500-normal.woff2"),
        )),
        _ => None,
    }
}

/// Navigation is allowed only to the exact origin `easy-note://localhost` (not another host on the
/// scheme, not `easy-note://localhost.evil/`, not a port or user part).
pub fn allowed_url(url: &str) -> bool {
    url.strip_prefix(ORIGIN)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
}

/// The script that hands one `api::handle` answer to the page: the JSON goes in as one JS string
/// literal (serde_json escapes it), never spliced in raw.
pub fn reply_script(answer: &str) -> String {
    format!(
        "window.__easyNote.reply({})",
        Value::String(answer.to_string())
    )
}

/// The initialization script for `dashboard --note <id>`: it runs before `ipc.js`, which keeps
/// `initialNote`. The id goes in as one JSON string (serde_json escapes it), never spliced in raw.
pub fn initial_note_script(id: &str) -> String {
    format!(
        "window.__easyNote = {};",
        serde_json::json!({ "initialNote": id })
    )
}

/// What the shell runs when a second `dashboard --note <id>` call arrives (HUN-290): the page shows
/// that note. The id goes in as one JSON string, never spliced in raw.
pub fn show_script(id: &str) -> String {
    format!(
        "window.__easyNote && window.__easyNote.show && window.__easyNote.show({})",
        Value::String(id.to_string())
    )
}

/// One IPC message from the page: `{"req": <number>, "cmd": ..., ...}`. `req` is taken out, the
/// rest goes to `api::handle`, and the same `req` is put back at the top level of the answer
/// (an error too), so the page matches each reply to its request. A `req` that is not a number
/// is dropped, never echoed.
pub fn ipc(store: &Store, body: &str) -> String {
    ipc_with(store, body, &|_| {})
}

/// `ipc`, with the `api::handle_with` hook for the id of a saved note.
pub fn ipc_with(store: &Store, body: &str, on_added: &dyn Fn(&str)) -> String {
    let Ok(mut request) = serde_json::from_str::<Value>(body) else {
        return api::handle_with(store, body, on_added);
    };
    let req = request
        .as_object_mut()
        .and_then(|o| o.remove("req"))
        .filter(Value::is_number);
    let answer = api::handle_with(store, &request.to_string(), on_added);
    let Some(req) = req else { return answer };
    match serde_json::from_str::<Value>(&answer) {
        Ok(Value::Object(mut map)) => {
            map.insert("req".to_string(), req);
            Value::Object(map).to_string()
        }
        _ => answer,
    }
}

/// One IPC message with its source: wry gives the sending page's URL as the request URI (on
/// WebKitGTK the main frame URL). A message from any page outside `ORIGIN` gets no answer.
pub fn ipc_from(store: &Store, source: &str, body: &str) -> Option<String> {
    ipc_from_with(store, source, body, &|_| {})
}

/// `ipc_from`, with the `api::handle_with` hook for the id of a saved note.
pub fn ipc_from_with(
    store: &Store,
    source: &str,
    body: &str,
    on_added: &dyn Fn(&str),
) -> Option<String> {
    allowed_url(source).then(|| ipc_with(store, body, on_added))
}

/// The note ids with an `enrich` run in flight: one run per note at a time (HUN-288).
#[derive(Clone, Default)]
pub struct Running(Arc<Mutex<HashSet<String>>>);

/// A claimed id; the id is free again when this drops, a panic of the run included.
struct Slot(Running, String);

impl Running {
    fn start(&self, id: &str) -> Option<Slot> {
        let mut ids = self.0.lock().unwrap_or_else(|e| e.into_inner());
        ids.insert(id.to_string())
            .then(|| Slot(self.clone(), id.to_string()))
    }
}

impl Drop for Slot {
    fn drop(&mut self) {
        let mut ids = (self.0).0.lock().unwrap_or_else(|e| e.into_inner());
        ids.remove(&self.1);
    }
}

/// `ipc_from_with`, with the answer handed to `send` instead of returned, and `enrich` (it waits
/// for the person's `claude`, seconds) run on its own thread, so the caller never waits for it:
/// the window keeps answering `board`, `detail` and the focus refresh meanwhile (HUN-288). `send`
/// gets each answer once; a request from a page outside `ORIGIN` gets none. A second `enrich` of
/// a note that is running is answered at once with an error and starts no child.
pub fn ipc_threaded(
    store: &Store,
    running: &Running,
    source: &str,
    body: &str,
    on_added: &dyn Fn(&str),
    send: impl Fn(String) + Send + 'static,
) {
    if !allowed_url(source) {
        return;
    }
    let request = serde_json::from_str::<Value>(body).unwrap_or(Value::Null);
    let id = request
        .get("id")
        .and_then(Value::as_str)
        .filter(|_| request.get("cmd").and_then(Value::as_str) == Some("enrich"));
    let Some(id) = id else {
        return send(ipc_with(store, body, on_added));
    };
    let Some(slot) = running.start(id) else {
        let mut answer = serde_json::json!({ "error": "AI is already running for this note" });
        if let Some(req) = request.get("req").filter(|r| r.is_number()) {
            answer["req"] = req.clone();
        }
        return send(answer.to_string());
    };
    let (store, body) = (store.clone(), body.to_string());
    let req = request.get("req").filter(|r| r.is_number()).cloned();
    std::thread::spawn(move || {
        // A panic must still answer, or the page's button stays at "AI running…".
        let answer = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ipc_with(&store, &body, &|_| {})
        }))
        .unwrap_or_else(|_| {
            let mut failed = serde_json::json!({ "error": "AI run failed" });
            if let Some(req) = req {
                failed["req"] = req;
            }
            failed.to_string()
        });
        drop(slot);
        send(answer);
    });
}

/// What the quick window's page asked (HUN-289). It talks `context`, `add` and `close` only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Capture {
    /// The answer to hand back to the page (`reply_script`), with the request's `req`.
    Reply(String),
    /// Enter on some text: `quick` saves it as it always did; the chip's context only with `keep_context`.
    Save { text: String, keep_context: bool },
    /// Esc, or Enter with no text: nothing is saved.
    Close,
}

/// One IPC message of the quick window. `context` is the one `quick` computed (the `--cwd` folder's,
/// else the last context): `{"context": {repo, task} | null}` as the dashboard's `context` answers.
/// `add` with text is a `Save` (no reply: the window closes), or `the text is too long` (the dialog and
/// its text stay, as in the dashboard); every other command is
/// `unknown command`. Every string in a reply passes `guard::mask_for_output`.
pub fn capture_ipc(body: &str, context: &Context) -> Capture {
    let Ok(request) = serde_json::from_str::<Value>(body) else {
        return Capture::Reply(serde_json::json!({ "error": "bad request" }).to_string());
    };
    let req = request.get("req").filter(|r| r.is_number()).cloned();
    let answer = match request.get("cmd").and_then(Value::as_str) {
        Some("close") => return Capture::Close,
        Some("add") => match request.get("text").and_then(Value::as_str) {
            Some(text) if !text.trim().is_empty() && check_text(text.trim()).is_err() => {
                serde_json::json!({ "error": "the text is too long" })
            }
            Some(text) if !text.trim().is_empty() => {
                let keep_context =
                    request.get("keep_context").and_then(Value::as_bool) == Some(true);
                return Capture::Save {
                    text: text.to_string(),
                    keep_context,
                };
            }
            Some(_) => serde_json::json!({ "error": "empty" }),
            None => serde_json::json!({ "error": "missing text" }),
        },
        Some("context") => {
            let chip = (context.repo.is_some() || context.task.is_some()).then(|| {
                let mask = |v: &Option<String>| v.as_deref().map(guard::mask_for_output);
                serde_json::json!({ "repo": mask(&context.repo), "task": mask(&context.task) })
            });
            serde_json::json!({ "context": chip })
        }
        Some(_) => serde_json::json!({ "error": "unknown command" }),
        None => serde_json::json!({ "error": "missing cmd" }),
    };
    let mut answer = answer;
    if let (Some(req), Value::Object(map)) = (req, &mut answer) {
        map.insert("req".to_string(), req);
    }
    Capture::Reply(answer.to_string())
}

/// `capture_ipc` for a message from `source`; a page outside `ORIGIN` gets nothing (as `ipc_from`).
pub fn capture_from(source: &str, body: &str, context: &Context) -> Option<Capture> {
    allowed_url(source).then(|| capture_ipc(body, context))
}

/// `open`'s error when the display cannot be used (GTK does not start): `dashboard` then takes
/// the FR-034 Obsidian path instead (HUN-281).
pub const NO_DISPLAY: &str = "the display cannot be used";

/// Without the feature, `dashboard` never asks for the window (`dashboard::use_window`).
#[cfg(not(feature = "window"))]
pub fn open(
    _initial_note: Option<&str>,
    _owner: single::Owner,
    _on_shown: impl FnOnce(),
) -> Result<(), String> {
    Err("this easy-note is built without the window feature".to_string())
}

/// Without the feature, `quick` never asks for the window (`dashboard::window_here`).
#[cfg(not(feature = "window"))]
pub fn open_capture(_context: &Context) -> Result<Option<(String, bool)>, String> {
    Err("this easy-note is built without the window feature".to_string())
}

#[cfg(feature = "window")]
pub use shell::{open, open_capture};

#[cfg(feature = "window")]
mod shell {
    use std::borrow::Cow;
    use std::time::{Duration, Instant};

    use tao::dpi::{LogicalPosition, LogicalSize};
    use tao::event::{Event, WindowEvent};
    use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
    use tao::platform::run_return::EventLoopExtRunReturn;
    use tao::platform::unix::WindowExtUnix;
    use tao::window::{UserAttentionType, WindowBuilder};
    use wry::http::{header, Response, StatusCode};
    use wry::{NewWindowResponse, WebViewBuilder, WebViewBuilderExtUnix};

    use super::{
        allowed_url, asset, capture_from, initial_note_script, ipc_threaded, reply_script,
        show_script, single, Capture, Running, CSP, QUICK_START, REFRESH, START,
    };
    use crate::core::config;
    use crate::core::context::Context;
    use crate::core::store::Store;
    use crate::surfaces::enrich;

    /// Some GPU / Wayland setups draw a blank WebKitGTK window without it (DDR 0009).
    const DMABUF: &str = "WEBKIT_DISABLE_DMABUF_RENDERER";

    /// The person's own value wins; set ours only when the variable is unset.
    pub(super) fn needs_dmabuf_default(current: Option<&std::ffi::OsStr>) -> bool {
        current.is_none()
    }

    /// The quick window's size: the design dialog (560 px) and its 16 px margin (quick.css).
    const QUICK_SIZE: (f64, f64) = (592.0, 226.0);
    /// How long the loop keeps running after the hide, so GTK sends it to the display server before
    /// `quick` saves and notifies (a hide that is only queued would leave the window up meanwhile).
    const HIDE_FLUSH: Duration = Duration::from_millis(60);

    /// Build the event loop; tao panics ("Failed to initialize gtk backend!") when GTK cannot open
    /// the display, for example `DISPLAY` set to a display that is gone. Catch that one panic, quietly.
    fn event_loop<T: 'static>() -> Result<EventLoop<T>, String> {
        if needs_dmabuf_default(std::env::var_os(DMABUF).as_deref()) {
            // Before any GTK or WebKit code runs; still one thread here.
            std::env::set_var(DMABUF, "1");
        }
        let quiet = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let built = std::panic::catch_unwind(|| EventLoopBuilder::<T>::with_user_event().build());
        std::panic::set_hook(quiet);
        built.map_err(|_| super::NO_DISPLAY.to_string())
    }

    /// The quick window (HUN-289): the capture dialog alone, centred, until Enter, Esc or close.
    /// `Ok(Some((text, keep_context)))` when the person pressed Enter on some text, `Ok(None)` on a
    /// cancel; `Err` when the window cannot open (`quick` then shows the zenity box). It saves
    /// nothing: `quick` does, the way it always did.
    pub fn open_capture(context: &Context) -> Result<Option<(String, bool)>, String> {
        let mut event_loop = event_loop::<Capture>()?;
        let proxy = event_loop.create_proxy();
        let (width, height) = QUICK_SIZE;
        let mut builder = WindowBuilder::new()
            .with_title("easy-note — new note")
            .with_inner_size(LogicalSize::new(width, height))
            .with_resizable(false)
            .with_always_on_top(true)
            .with_focused(true);
        // X11 takes the position; on Wayland the compositor places the window itself.
        if let Some(monitor) = event_loop.primary_monitor() {
            let scale = monitor.scale_factor();
            let size = monitor.size().to_logical::<f64>(scale);
            let at = monitor.position().to_logical::<f64>(scale);
            builder = builder.with_position(LogicalPosition::new(
                at.x + (size.width - width) / 2.0,
                at.y + (size.height - height) / 3.0,
            ));
        }
        let window = builder.build(&event_loop).map_err(|e| e.to_string())?;
        let vbox = window
            .default_vbox()
            .ok_or_else(|| "no GTK container for the webview".to_string())?;
        let context = context.clone();
        let webview = WebViewBuilder::new()
            .with_custom_protocol("easy-note".into(), |_, request| {
                respond(request.uri().path())
            })
            .with_ipc_handler(move |request| {
                let source = request.uri().to_string();
                if let Some(message) = capture_from(&source, request.body(), &context) {
                    let _ = proxy.send_event(message);
                }
            })
            // The quick page only: not even the dashboard's page on the same origin.
            .with_navigation_handler(|url| url == QUICK_START)
            .with_new_window_req_handler(|_, _| NewWindowResponse::Deny)
            .with_devtools(cfg!(debug_assertions))
            .with_url(QUICK_START)
            .build_gtk(vbox)
            .map_err(|e| e.to_string())?;
        window.set_focus();
        let mut answer = None;
        let mut closing: Option<Instant> = None;
        event_loop.run_return(|event, _, flow| {
            if let Some(until) = closing {
                // Hidden already: run until the hide is out, then stop. Later messages change nothing.
                *flow = if Instant::now() >= until {
                    ControlFlow::Exit
                } else {
                    ControlFlow::WaitUntil(until)
                };
                return;
            }
            *flow = ControlFlow::Wait;
            let done = match event {
                Event::UserEvent(Capture::Reply(reply)) => {
                    let _ = webview.evaluate_script(&reply_script(&reply));
                    false
                }
                Event::UserEvent(Capture::Save { text, keep_context }) => {
                    answer = Some((text, keep_context));
                    true
                }
                Event::UserEvent(Capture::Close)
                | Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => true,
                _ => false,
            };
            if done {
                // Gone from the screen first: the save and its notification come after.
                window.set_visible(false);
                let until = Instant::now() + HIDE_FLUSH;
                closing = Some(until);
                *flow = ControlFlow::WaitUntil(until);
            }
        });
        drop(webview);
        drop(window);
        Ok(answer)
    }

    fn respond(path: &str) -> Response<Cow<'static, [u8]>> {
        let (status, mime, body): (StatusCode, &str, &'static [u8]) = match asset(path) {
            Some((mime, body)) => (StatusCode::OK, mime, body),
            None => (StatusCode::NOT_FOUND, "text/plain; charset=utf-8", b""),
        };
        Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, mime)
            .header(header::CONTENT_SECURITY_POLICY, CSP)
            .body(Cow::Borrowed(body))
            .unwrap_or_else(|_| Response::new(Cow::Borrowed(b"")))
    }

    /// What reaches the dashboard's event loop: an answer for the page, or a request from a second call.
    enum Ui {
        Answer(String),
        Focus(Option<String>),
    }

    /// Open the window and run its event loop until it is closed (the process then exits).
    /// `owner` holds the window lock and serves the focus requests of a second call (HUN-290).
    /// `initial_note` is the id the page selects after its first board load (`dashboard --note`).
    /// `on_shown` runs once, after the window and webview exist and before the loop starts; it
    /// does not run when the display cannot be used.
    pub fn open(
        initial_note: Option<&str>,
        owner: single::Owner,
        on_shown: impl FnOnce(),
    ) -> Result<(), String> {
        let store = Store::system(config::notes_dir().map_err(|e| e.to_string())?);
        let event_loop = event_loop::<Ui>()?;
        let proxy = event_loop.create_proxy();
        let running = Running::default();
        let window = WindowBuilder::new()
            .with_title("easy-note")
            .with_inner_size(LogicalSize::new(1280.0, 820.0))
            .with_min_inner_size(LogicalSize::new(800.0, 560.0))
            .build(&event_loop)
            .map_err(|e| e.to_string())?;
        let vbox = window
            .default_vbox()
            .ok_or_else(|| "no GTK container for the webview".to_string())?;
        let mut builder = WebViewBuilder::new();
        if let Some(id) = initial_note {
            builder = builder.with_initialization_script(initial_note_script(id));
        }
        let webview = builder
            .with_custom_protocol("easy-note".into(), |_, request| {
                respond(request.uri().path())
            })
            .with_ipc_handler(move |request| {
                // The answer goes back through the event loop, where the webview lives.
                let source = request.uri().to_string();
                let on_added = |id: &str| {
                    // The `[ai] enrich` + `after_save` rule of `easy-note add`, read at each
                    // save: the Settings screen may have changed it since the window opened.
                    if config::after_save_on() {
                        enrich::after_save(&store.root, id);
                    }
                };
                // `enrich` runs on its own thread and answers through the proxy when the child ends.
                let proxy = proxy.clone();
                ipc_threaded(
                    &store,
                    &running,
                    &source,
                    request.body(),
                    &on_added,
                    move |a| {
                        let _ = proxy.send_event(Ui::Answer(a));
                    },
                );
            })
            .with_navigation_handler(|url| allowed_url(&url))
            .with_new_window_req_handler(|_, _| NewWindowResponse::Deny)
            .with_devtools(cfg!(debug_assertions))
            .with_url(START)
            .build_gtk(vbox)
            .map_err(|e| e.to_string())?;
        let focus = event_loop.create_proxy();
        owner.serve(move |note| {
            let _ = focus.send_event(Ui::Focus(note));
        });
        on_shown();
        event_loop.run(move |event, _, flow| {
            *flow = ControlFlow::Wait;
            match event {
                Event::UserEvent(Ui::Answer(answer)) => {
                    let _ = webview.evaluate_script(&reply_script(&answer));
                }
                Event::UserEvent(Ui::Focus(note)) => {
                    window.set_minimized(false);
                    window.set_visible(true);
                    window.set_focus();
                    // Where the window manager refuses the focus, this still marks the window.
                    window.request_user_attention(Some(UserAttentionType::Informational));
                    if let Some(id) = note {
                        let _ = webview.evaluate_script(&show_script(&id));
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::Focused(true),
                    ..
                } => {
                    let _ = webview.evaluate_script(REFRESH);
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *flow = ControlFlow::Exit,
                _ => {}
            }
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::ffi::OsStr;

        #[test]
        fn dmabuf_is_set_only_when_unset() {
            assert!(needs_dmabuf_default(None));
            assert!(!needs_dmabuf_default(Some(OsStr::new("0"))));
            assert!(!needs_dmabuf_default(Some(OsStr::new("1"))));
            assert!(!needs_dmabuf_default(Some(OsStr::new(""))));
        }

        #[test]
        fn every_response_carries_the_csp() {
            for path in ["/", "/app.js", "/app.css", "/ipc.js", "/missing"] {
                let r = respond(path);
                assert_eq!(r.headers()[header::CONTENT_SECURITY_POLICY], CSP, "{path}");
            }
            assert_eq!(respond("/").status(), StatusCode::OK);
            assert_eq!(respond("/missing").status(), StatusCode::NOT_FOUND);
        }
    }
}
