/*
 * IPC contract between this page and the Rust window (HUN-278). All of it lives in this file.
 *
 * Request (page -> Rust):
 *   window.ipc.postMessage(JSON.stringify({ req: <number>, cmd: <string>, ...args }))
 *   - req: a number that increases by 1 for each request of this page (1, 2, 3, ...)
 *   - the rest is exactly the `window::api::handle` request:
 *       { "req": 1, "cmd": "board" }
 *       { "req": 2, "cmd": "detail", "id": "20261119-104212-3f0a" }
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
 *               notes: [{ id, created, time, day, first, text, repo, task, tags, source, kind, ai, stale }] }
 *   detail -> the note item plus { masked, context, ai: { title, kind, summary, tags_suggested, model, at, stale } | null }
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
    detail: function (noteId) { return request('detail', { id: noteId }); }
  };
})();
