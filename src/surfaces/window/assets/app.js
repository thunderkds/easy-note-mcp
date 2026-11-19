'use strict';
// Placeholder page logic (HUN-278 backend slice). Every note string is set with textContent.
// The shell answers each IPC request by calling window.__easyNote.reply(<json text>); the answer
// carries the request's `req` number.
(function () {
  const waiting = new Map();
  let next = 1;
  let loading = false;

  function ask(request, done) {
    const req = next++;
    waiting.set(req, done);
    request.req = req;
    window.ipc.postMessage(JSON.stringify(request));
  }

  function el(tag, text) {
    const node = document.createElement(tag);
    if (text !== undefined && text !== null) node.textContent = String(text);
    return node;
  }

  function load() {
    if (loading) return;
    loading = true;
    ask({ cmd: 'board' }, function (board) {
      loading = false;
      render(board);
    });
  }

  function render(board) {
    const counts = document.getElementById('counts');
    if (board.error) {
      counts.textContent = board.error;
      return;
    }
    document.getElementById('dir').textContent = board.notes_dir;
    counts.textContent = board.today + ' today · ' + board.total + ' notes · ' + board.projects + ' projects';
    const list = document.createDocumentFragment();
    board.notes.forEach(function (n) {
      const li = el('li');
      li.append(el('span', n.time + ' '), el('span', n.first));
      if (n.kind) li.append(el('span', ' [' + n.kind + ']'));
      if (n.ai) li.append(el('span', n.stale ? ' stale' : ' AI'));
      li.addEventListener('click', function () { show(n.id); });
      list.append(li);
    });
    document.getElementById('notes').replaceChildren(list);
  }

  function show(id) {
    ask({ cmd: 'detail', id: id }, function (d) {
      const box = document.getElementById('detail');
      box.hidden = false;
      if (d.error) {
        box.replaceChildren(el('p', d.error));
        return;
      }
      const ai = d.ai
        ? el('p', 'AI detail (not your words): ' + d.ai.title + ' — ' + d.ai.summary)
        : el('p', 'No AI detail yet.');
      box.replaceChildren(el('h2', d.id), el('pre', d.text), ai);
    });
  }

  window.__easyNote = {
    reply: function (text) {
      const answer = JSON.parse(text);
      const done = waiting.get(answer.req);
      waiting.delete(answer.req);
      if (done) done(answer);
    },
    refresh: load,
  };
  load();
})();
