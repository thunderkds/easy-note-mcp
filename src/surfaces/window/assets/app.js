/* easy-note dashboard page (read-only, HUN-278). Port of docs/design/Main.dc.html, board "Dashboard (clickable)".
 * Every note-derived string goes into the DOM with textContent / createElement only. */
(function () {
  'use strict';

  var DAY_LABELS = { today: 'today', yesterday: 'yesterday', earlier: 'earlier this week' };
  var FILTER_DEFS = [
    ['all', 'All', function () { return true; }],
    ['bug', 'Bugs', function (n) { return n.ai && n.kind === 'bug'; }],
    ['idea', 'Ideas', function (n) { return n.ai && n.kind === 'idea'; }],
    ['question', 'Questions', function (n) { return n.ai && n.kind === 'question'; }],
    ['to-do', 'To-dos', function (n) { return n.ai && n.kind === 'to-do'; }],
    ['noai', 'No AI detail', function (n) { return !n.ai; }],
    ['stale', 'Stale', function (n) { return !!(n.ai && n.stale); }]
  ];

  var state = { board: null, notes: [], query: '', filter: 'all', sel: '', detail: null, detailError: '' };

  function $(id) { return document.getElementById(id); }

  function el(tag, cls, text) {
    var e = document.createElement(tag);
    if (cls) e.className = cls;
    if (text !== undefined && text !== null) e.textContent = String(text);
    return e;
  }

  function icon(id) { return document.getElementById(id).content.firstElementChild.cloneNode(true); }

  function str(v) { return v === undefined || v === null ? '' : String(v); }
  function tagsOf(n) { return Array.isArray(n.tags) ? n.tags.map(str) : []; }

  // Row time: "HH:MM" for today / yesterday; for earlier notes the weekday ("Mon") within a week, else "17 Nov".
  function rowTime(n) {
    if (n.day !== 'earlier') return str(n.time);
    var d = new Date(str(n.created));
    if (isNaN(d.getTime())) return str(n.time);
    if (Date.now() - d.getTime() < 7 * 24 * 3600 * 1000) return d.toLocaleDateString('en-GB', { weekday: 'short' });
    return d.toLocaleDateString('en-GB', { day: 'numeric', month: 'short' });
  }

  function shortDate(v) {
    var d = new Date(str(v));
    return isNaN(d.getTime()) ? str(v) : d.toLocaleDateString('en-GB', { day: 'numeric', month: 'short' });
  }

  function kindLabel(k) { return k === 'todo' ? 'to-do' : str(k); }

  function matches(n, words) {
    var hay = (str(n.text) + ' ' + str(n.repo) + ' ' + str(n.task) + ' ' + tagsOf(n).join(' ')).toLowerCase();
    return words.every(function (w) { return hay.indexOf(w) >= 0; });
  }

  // ---- counts strip and header ----
  function renderCounts() {
    var b = state.board || {};
    $('notes-dir').textContent = str(b.notes_dir);
    $('count-today').textContent = str(b.today || 0);
    $('count-total').textContent = str(b.total || 0);
    $('count-projects').textContent = str(b.projects || 0);
    var box = $('by-project');
    var label = box.firstElementChild;
    box.replaceChildren(label);
    var top = Array.isArray(b.top_projects) ? b.top_projects.slice(0, 3) : [];
    var max = top.length && top[0].n > 0 ? top[0].n : 1;
    top.forEach(function (r) {
      var row = el('div', 'bar-row');
      row.appendChild(el('span', 'bar-name', r.name));
      var track = el('span', 'bar-track');
      var fill = el('span', 'bar-fill');
      fill.style.width = Math.round((Number(r.n) / max) * 100) + '%';
      track.appendChild(fill);
      row.appendChild(track);
      row.appendChild(el('span', 'bar-n', r.n));
      box.appendChild(row);
    });
  }

  // ---- filters, search, day groups ----
  var filterButtons = [];
  var rowButtons = Object.create(null);

  function buildFilters() {
    var box = $('filters');
    FILTER_DEFS.forEach(function (f) {
      var b = el('button', 'filter', f[1] + ' ');
      b.type = 'button';
      b.setAttribute('aria-pressed', 'false');
      b.appendChild(el('span', 'filter-n', '0'));
      b.addEventListener('click', function () { state.filter = f[0]; renderList(); });
      box.appendChild(b);
      filterButtons.push(b);
    });
  }

  function renderList() {
    var notes = state.notes;
    var words = state.query.toLowerCase().split(/\s+/).filter(Boolean);
    var active = FILTER_DEFS.filter(function (f) { return f[0] === state.filter; })[0] || FILTER_DEFS[0];

    // Filter buttons are built once (buildFilters) and updated in place, so keyboard focus stays on them.
    FILTER_DEFS.forEach(function (f, i) {
      var b = filterButtons[i];
      b.setAttribute('aria-pressed', f[0] === state.filter ? 'true' : 'false');
      b.lastChild.textContent = String(notes.filter(f[2]).length);
    });

    var shown = notes.filter(function (n) { return active[2](n) && matches(n, words); });

    var none = $('no-results');
    none.hidden = shown.length > 0 || !state.board;
    var q = state.query.trim();
    $('no-results-text').textContent = !notes.length
      ? 'No notes yet. Your first one appears here.'
      : q ? 'No note matches “' + state.query + '”.' : 'No note matches.';
    none.querySelector('.empty-hint').hidden = !q;
    $('clear-search').hidden = !q;

    rowButtons = Object.create(null);
    var frag = document.createDocumentFragment();
    ['today', 'yesterday', 'earlier'].forEach(function (d) {
      var rows = shown.filter(function (n) { return n.day === d; });
      if (!rows.length) return;
      var g = el('div', 'group');
      g.appendChild(el('h2', 'group-label', '// ' + DAY_LABELS[d]));
      var ul = el('ul', 'rows');
      rows.forEach(function (n) { ul.appendChild(renderRow(n)); });
      g.appendChild(ul);
      frag.appendChild(g);
    });
    $('groups').replaceChildren(frag);
  }

  function renderRow(n) {
    var li = el('li');
    var b = el('button', 'row');
    b.type = 'button';
    b.setAttribute('aria-current', n.id === state.sel ? 'true' : 'false');
    b.appendChild(el('span', 'row-time', rowTime(n)));
    var body = el('span', 'row-body');
    body.appendChild(el('span', 'row-first', n.first !== undefined ? n.first : str(n.text).split('\n')[0]));
    var tags = tagsOf(n).map(function (t) { return '#' + t; }).join(' ');
    body.appendChild(el('span', 'row-where', [str(n.repo) || 'no project', str(n.task), tags].filter(Boolean).join(' · ')));
    b.appendChild(body);
    if (n.ai && n.kind) b.appendChild(el('span', 'chip', kindLabel(n.kind)));
    b.appendChild(el('span', 'row-mark' + (n.ai && n.stale ? ' stale' : ''), n.ai ? (n.stale ? 'stale' : 'AI') : ''));
    b.addEventListener('click', function () { select(n.id); });
    rowButtons[n.id] = b;
    li.appendChild(b);
    return li;
  }

  // ---- detail panel ----
  function renderDetail() {
    var aside = $('detail');
    var d = state.detail;
    if (!d) {
      var p = el('p', 'pick', state.detailError ? 'Could not read the note: ' + state.detailError : 'Pick a note to read it.');
      aside.replaceChildren(p);
      return;
    }
    var card = el('article', 'card');

    var head = el('div', 'card-head');
    head.appendChild(el('span', 'card-id', d.id));
    head.appendChild(el('span', 'chip', d.source));
    card.appendChild(head);

    card.appendChild(el('p', 'words-label', 'Your words'));
    card.appendChild(el('p', 'words', d.text));
    if (d.masked) card.appendChild(el('p', 'masked', 'secret masked'));

    var ctx = d.context;
    var branch = typeof ctx === 'string' ? ctx : ctx && ctx.branch;
    var when = d.day === 'today' ? 'today ' + str(d.time) : d.day === 'yesterday' ? 'yesterday ' + str(d.time) : rowTime(d);
    card.appendChild(el('p', 'context', [str(d.repo) || 'no project', str(branch), str(d.task), when].filter(Boolean).join(' · ')));

    var tagBox = el('div', 'tags');
    var tags = tagsOf(d);
    tags.forEach(function (t) { tagBox.appendChild(el('span', 'tag', '#' + t)); });
    if (!tags.length) tagBox.appendChild(el('span', 'no-tags', 'no tags'));
    card.appendChild(tagBox);

    if (d.ai) card.appendChild(renderAi(d.ai));
    else {
      var none = el('p', 'no-ai', 'No AI detail yet. In Claude Code, run the ');
      none.appendChild(el('code', null, 'enrich_notes'));
      none.appendChild(document.createTextNode(' prompt; your own AI writes it in a separate file.'));
      card.appendChild(none);
    }
    aside.replaceChildren(card);
  }

  function renderAi(ai) {
    var sec = el('section', 'ai');
    sec.setAttribute('aria-label', 'AI detail, not your words');
    var head = el('div', 'ai-head');
    head.appendChild(icon('icon-ai'));
    head.appendChild(el('strong', 'ai-label', 'AI detail (not your words)'));
    head.appendChild(el('span', 'ai-by', [str(ai.model), shortDate(ai.at)].filter(Boolean).join(' · ')));
    sec.appendChild(head);
    if (ai.stale) {
      var st = el('p', 'ai-stale');
      st.appendChild(icon('icon-stale'));
      st.appendChild(document.createTextNode('stale: you edited the note after the AI wrote this'));
      sec.appendChild(st);
    }
    sec.appendChild(el('p', 'ai-title', ai.title));
    sec.appendChild(el('p', 'ai-kind', 'type: ' + kindLabel(ai.kind)));
    sec.appendChild(el('p', 'ai-summary', ai.summary));
    var sug = Array.isArray(ai.tags_suggested) ? ai.tags_suggested : [];
    if (sug.length) {
      var row = el('div', 'ai-suggest');
      row.appendChild(el('span', 'ai-suggest-label', 'suggested'));
      sug.forEach(function (t) { row.appendChild(el('span', 'ai-tag', '#' + str(t))); });
      sec.appendChild(row);
    }
    return sec;
  }

  // ---- data ----
  function showError(msg) {
    var e = $('error');
    e.textContent = msg ? 'Could not load notes: ' + msg : '';
    e.hidden = !msg;
  }

  function loadBoard() {
    return window.easyNoteIpc.board().then(function (b) {
      state.board = b;
      state.notes = Array.isArray(b.notes) ? b.notes : [];
      showError('');
      if (state.sel && !state.notes.some(function (n) { return n.id === state.sel; })) {
        state.sel = '';
        state.detail = null;
        renderDetail();
      }
      renderCounts();
      renderList();
    }, function (err) {
      showError(err.message);
    });
  }

  function select(id) {
    state.sel = id;
    state.detailError = '';
    Object.keys(rowButtons).forEach(function (k) { rowButtons[k].setAttribute('aria-current', k === id ? 'true' : 'false'); });
    window.easyNoteIpc.detail(id).then(function (d) {
      if (state.sel !== id) return;
      state.detail = d;
      renderDetail();
    }, function (err) {
      if (state.sel !== id) return;
      state.detail = null;
      state.detailError = err.message;
      renderDetail();
    });
  }

  $('search').addEventListener('input', function (e) { state.query = e.target.value; renderList(); });
  $('clear-search').addEventListener('click', function () {
    state.query = '';
    $('search').value = '';
    renderList();
  });
  // The Rust shell calls window.__easyNote.refresh() on each window focus (mod.rs REFRESH).
  window.__easyNote.refresh = loadBoard;
  buildFilters();
  renderList();
  loadBoard();
})();
