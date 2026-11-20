/*
 * IPC contract between this page and the Rust window (HUN-278 read, HUN-279 actions). All of it lives in this file.
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
 *       { "req": 6, "cmd": "open_in_editor", "id": "<note id>" }
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
 *   accept_ai      -> the updated `detail` (the AI tags_suggested added to the note's tags, `easy-note tag <id> --accept-ai`)
 *                     errors: "no AI detail", "no such note", "bad id", "missing id", "inbox notes are read-only",
 *                     "cannot use the note"
 *   trash          -> { trashed: <id> } (`easy-note rm`; the AI file moves too)
 *                     errors: "no such note", "bad id", "missing id", "a file of that name is in the way",
 *                     "cannot use the note"
 *   restore        -> { restored: <id> } (`easy-note restore`; the page's Undo)
 *                     errors: as trash; "no such note" also when the note is already back
 *   open_in_editor -> { opened: <id> } (`xdg-open <note file>`, an argv list, no shell)
 *                     errors: "no such note", "bad id", "missing id", "xdg-open failed", "cannot run xdg-open",
 *                     "cannot read the note"
 *   A trashed id is "no such note" for detail, accept_ai and open_in_editor.
 *   Copy id needs no request: the page uses navigator.clipboard.
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

  window.__easyNote = { reply: reply };
  window.easyNoteIpc = {
    board: function () { return request('board'); },
    detail: function (noteId) { return request('detail', { id: noteId }); },
    acceptAi: function (noteId) { return request('accept_ai', { id: noteId }); },
    trash: function (noteId) { return request('trash', { id: noteId }); },
    restore: function (noteId) { return request('restore', { id: noteId }); },
    openInEditor: function (noteId) { return request('open_in_editor', { id: noteId }); }
  };
})();
