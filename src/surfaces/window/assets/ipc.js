/*
 * IPC contract between this page and the Rust window (HUN-278 read, HUN-279 actions, HUN-280 capture,
 * HUN-286 edit and the initial note, HUN-288 Run AI). All of it lives in this file.
 *
 * Request (page -> Rust):
 *   window.ipc.postMessage(JSON.stringify({ req: <number>, cmd: <string>, ...args }))
 *   - req: a number that increases by 1 for each request of this page (1, 2, 3, ...)
 *   - the rest is exactly the `window::api::handle` request:
 *       { "req": 1, "cmd": "board" }
 *       { "req": 2, "cmd": "detail", "id": "20261119-104212-3f0a" }
 *       { "req": 3, "cmd": "accept_ai", "id": "<note id>" }
 *       { "req": 4, "cmd": "trash", "id": "<note id>" }
 *       { "req": 5, "cmd": "restore", "id": "<note id>" }
 *       { "req": 6, "cmd": "edit", "id": "<note id>", "text": "<the whole new text>",
 *         "tags": ["<tag>", ...], "base": "<the `text` of the detail the page loaded>",
 *         "base_tags": ["<the `tags` of that detail>", ...] }
 *       { "req": 7, "cmd": "context" }
 *       { "req": 8, "cmd": "add", "text": "<the typed text>", "keep_context": true }
 *       { "req": 9, "cmd": "enrich", "id": "<note id>" }
 *       { "req": 9, "cmd": "close" }   (the quick window only, HUN-289)
 *   The correlation key is `req`, not `id`, because `id` is already the note id in the `detail`
 *   request and in every note object of a reply.
 *
 * Reply (Rust -> page):
 *   Rust calls window.__easyNote.reply(<jsonString>), where <jsonString> is one JS string literal
 *   (serde_json::to_string of the response JSON text). The response is the `handle` output object
 *   with the request's `req` added at the top level:
 *   - success: { "req": 2, ...handle output }      -> the promise resolves with the whole object
 *   - failure: { "req": 2, "error": "<short text>" } -> the promise rejects with Error(<short text>)
 *   A reply whose `req` matches no pending request is ignored.
 *
 * Results (handle output):
 *   board  -> { notes_dir, today, total, projects, top_projects: [{ name, n }],
 *               notes: [{ id, created, time, day, first, search, repo, task, tags, source, kind, ai, stale }] }
 *               `search` is the masked body's distinct words (the board has no full text, HUN-282)
 *   detail -> the note item plus { text, masked, context, ai: { title, kind, summary, tags_suggested, model, at, stale } | null }
 *   An edited note also has { updated, edited } in both (`edited` is `2h ago`, or the date and time), and
 *   { edited_at } (the full local stamp) in `detail` (HUN-291); a note never edited has none of them.
 *   accept_ai      -> the updated `detail` (the AI tags_suggested added to the note's tags, `easy-note tag <id> --accept-ai`)
 *                     errors: "no AI detail", "no such note", "bad id", "missing id", "inbox notes are read-only",
 *                     "cannot use the note"
 *   trash          -> { trashed: <id> } (`easy-note rm`; the AI file moves too)
 *                     errors: "no such note", "bad id", "missing id", "a file of that name is in the way",
 *                     "cannot use the note"
 *   restore        -> { restored: <id> } (`easy-note restore`; the page's Undo)
 *                     errors: as trash; "no such note" also when the note is already back
 *   edit           -> the updated `detail` (`easy-note edit`: Store::update masks, re-validates and sets `updated`;
 *                     `masked` is true also when this save masked something). `text` and `tags` are the whole new
 *                     values, not a diff. `tags` follow `easy-note tag` (a leading `#` is dropped, each once, in
 *                     order; a tag the note already has is kept as it is); when the text changed, its `#words`
 *                     are added after them, as `easy-note edit` does. Same text (a trailing newline aside) and
 *                     same tags: nothing is written, the reply is the `detail`. The AI detail turns `stale` when
 *                     the text changed. No `after_save` enrich runs (`easy-note edit` runs none).
 *                     errors (nothing written): "the note changed outside the window" (the note's text is not
 *                     `base` now: reload the note with `detail`, keep the draft), "empty" (empty or white-space
 *                     text), "inbox notes are read-only", "no such note" (unknown or trashed id), "bad id",
 *                     "not a tag", "missing id", "missing text", "missing tags" (not a list of strings),
 *                     "missing base", "the text is too long", "cannot use the note"
 *   context        -> { context: { repo, task } | null } (the `easy-note quick` last context, FR-027); never fails
 *   enrich         -> the updated `detail`, its AI detail filled in (`easy-note ai enrich <id>`, `Run AI`, HUN-288):
 *                     the person's own `claude` runs, so the reply comes seconds later (up to the `[ai] timeout_secs`,
 *                     120 by default); the window answers every other request meanwhile. The click is the opt-in:
 *                     no `[ai]` section is needed. The note file is not changed. One run per note at a time.
 *                     errors (nothing written): words the person can act on, e.g. "claude not found (claude): install it,
 *                     or set [ai] command in <config.toml>", "claude exited 1; if it is not logged in, log in once: claude",
 *                     "timeout", "the answer is not JSON", "no such note", "missing id",
 *                     "AI is already running for this note"
 *   add            -> the new note's `detail` (`easy-note add` with `source: quick`; the last context when
 *                     keep_context is true, else none; a secret is masked on write)
 *                     errors: "empty", "missing text", "the text is too long", "cannot use the note"
 *   close          -> no reply: the quick window closes, nothing saved. The dashboard answers "unknown command".
 *
 * The quick window (`easy-note quick`, HUN-289) answers only `context`, `add` and `close`; any other
 * command is "unknown command". Its `context` is the chip `quick` computed (the `--cwd` folder's, or
 * the last context); its `add` gets no reply either: Rust saves the text as `quick` does and closes the
 * window.
 *   A trashed id is "no such note" for detail, accept_ai and edit.
 *   Copy id needs no request: the page uses navigator.clipboard.
 *   No request opens a note in another app (HUN-286 removed the xdg-open request).
 *
 * Initial note (Rust -> page, `easy-note dashboard --note <id>`, HUN-286):
 *   window.__easyNote.initialNote is the note id to select after the first board load, as a string,
 *   or null when the window was opened without `--note`. Rust checks the id with the store's id
 *   check first and hands it over in an initialization script that runs before this file:
 *     window.__easyNote = { initialNote: <serde_json string> };
 *   This file keeps that one value and adds `reply`. An unknown id gets the detail's
 *   "no such note" error, as any other id does.
 */
(function () {
  'use strict';
  var nextReq = 0;
  var pending = {};

  function reply(json) {
    var msg;
    try {
      msg = typeof json === 'string' ? JSON.parse(json) : json;
    } catch (e) {
      return;
    }
    if (!msg || typeof msg.req !== 'number' || !pending[msg.req]) return;
    var p = pending[msg.req];
    delete pending[msg.req];
    if (msg.error !== undefined) p.reject(new Error(String(msg.error)));
    else p.resolve(msg);
  }

  function request(cmd, args) {
    return new Promise(function (resolve, reject) {
      if (!window.ipc || typeof window.ipc.postMessage !== 'function') {
        reject(new Error('no IPC channel'));
        return;
      }
      nextReq += 1;
      var msg = { req: nextReq, cmd: cmd };
      if (args) {
        Object.keys(args).forEach(function (k) {
          if (k !== 'req' && k !== 'cmd') msg[k] = args[k];
        });
      }
      pending[nextReq] = { resolve: resolve, reject: reject };
      window.ipc.postMessage(JSON.stringify(msg));
    });
  }

  var prior = window.__easyNote;
  var initialNote = prior && typeof prior.initialNote === 'string' ? prior.initialNote : null;
  window.__easyNote = { reply: reply, initialNote: initialNote };
  window.easyNoteIpc = {
    board: function () { return request('board'); },
    detail: function (noteId) { return request('detail', { id: noteId }); },
    acceptAi: function (noteId) { return request('accept_ai', { id: noteId }); },
    trash: function (noteId) { return request('trash', { id: noteId }); },
    restore: function (noteId) { return request('restore', { id: noteId }); },
    edit: function (noteId, text, tags, base, baseTags) {
      return request('edit', { id: noteId, text: text, tags: tags, base: base, base_tags: baseTags });
    },
    context: function () { return request('context'); },
    add: function (text, keepContext) { return request('add', { text: text, keep_context: !!keepContext }); },
    enrich: function (noteId) { return request('enrich', { id: noteId }); },
    close: function () { return request('close'); }
  };
})();
