/* easy-note dashboard page (HUN-278 read, HUN-279 actions, HUN-280 capture). Port of docs/design/Main.dc.html, board "Dashboard (clickable)".
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
  var UNDO_MS = 6000;
  var SEARCH_MS = 120;  // HUN-282: the list renders this long after the last key press, not on each one
  var FRESH_MS = 1000;  // HUN-282: a focus refresh this soon after a board answer is the start-up focus, not news
  var TOAST_MS = 2800;

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

  // The board has no full text, only `search` (the body's distinct words); the haystack is built once per load.
  function hayOf(n) {
    return (str(n.search) + ' ' + str(n.repo) + ' ' + str(n.task) + ' ' + tagsOf(n).join(' ')).toLowerCase();
  }

  function matches(n, words) {
    return words.every(function (w) { return n.hay.indexOf(w) >= 0; });
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

  function shownNotes() {
    var words = state.query.toLowerCase().split(/\s+/).filter(Boolean);
    var active = FILTER_DEFS.filter(function (f) { return f[0] === state.filter; })[0] || FILTER_DEFS[0];
    return state.notes.filter(function (n) { return active[2](n) && matches(n, words); });
  }

  function renderList() {
    var notes = state.notes;

    // Filter buttons are built once (buildFilters) and updated in place, so keyboard focus stays on them.
    FILTER_DEFS.forEach(function (f, i) {
      var b = filterButtons[i];
      b.setAttribute('aria-pressed', f[0] === state.filter ? 'true' : 'false');
      b.lastChild.textContent = String(notes.filter(f[2]).length);
    });

    var shown = shownNotes();

    var none = $('no-results');
    none.hidden = shown.length > 0 || !state.board;
    var q = state.query.trim();
    $('no-results-text').textContent = !notes.length
      ? 'No notes yet. Your first one appears here.'
      : q ? 'No note matches “' + state.query + '”.' : 'No note matches.';
    none.querySelector('.empty-hint').hidden = !q;
    $('clear-search').hidden = !q;

    // Rows are rebuilt on each render (and each board reload), so keyboard focus moves to the new button of the same note.
    var focused = Object.keys(rowButtons).filter(function (k) { return rowButtons[k] === document.activeElement; })[0];
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
    if (focused) focusRow(focused);
  }

  function renderRow(n) {
    var li = el('li');
    var b = el('button', 'row');
    b.type = 'button';
    b.setAttribute('aria-current', n.id === state.sel ? 'true' : 'false');
    b.appendChild(el('span', 'row-time', rowTime(n)));
    var body = el('span', 'row-body');
    body.appendChild(el('span', 'row-first', str(n.first)));
    var tags = tagsOf(n).map(function (t) { return '#' + t; }).join(' ');
    body.appendChild(el('span', 'row-where', [str(n.repo) || 'no project', str(n.task), tags].filter(Boolean).join(' · ')));
    b.appendChild(body);
    if (n.ai && n.kind) b.appendChild(el('span', 'chip', kindLabel(n.kind)));
    b.appendChild(el('span', 'row-mark' + (n.ai && n.stale ? ' stale' : ''), n.ai ? (n.stale ? 'stale' : 'AI') : ''));
    b.addEventListener('click', function () { select(n.id, true); });
    rowButtons[n.id] = b;
    li.appendChild(b);
    return li;
  }

  // ---- detail panel ----
  // HUN-284: when the window is narrow the panel wraps below the list; after a row click the
  // rendered panel's top is brought into view. In two columns the panel is beside the list and
  // nothing scrolls.
  function revealDetail() {
    var aside = $('detail');
    if (aside.offsetTop <= aside.previousElementSibling.offsetTop) return;
    var still = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    aside.scrollIntoView({ block: 'start', behavior: still ? 'auto' : 'smooth' });
  }

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

    if (d.ai) card.appendChild(renderAi(d.ai, d));
    else {
      var none = el('p', 'no-ai', 'No AI detail yet. In Claude Code, run the ');
      none.appendChild(el('code', null, 'enrich_notes'));
      none.appendChild(document.createTextNode(' prompt; your own AI writes it in a separate file.'));
      card.appendChild(none);
    }

    var acts = el('div', 'actions');
    acts.appendChild(button('act', 'Copy id', function () { copyId(d.id); }));
    acts.appendChild(button('act danger', 'Move to trash', function () { trashNote(d.id); }));
    card.appendChild(acts);
    aside.replaceChildren(card);
  }

  function button(cls, label, onClick) {
    var b = el('button', cls, label);
    b.type = 'button';
    b.addEventListener('click', onClick);
    return b;
  }

  function renderAi(ai, d) {
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
      // Inbox notes are read-only (the CLI refuses `tag` on them too), so they get no button.
      var tags = tagsOf(d);
      var missing = sug.map(str).filter(function (t) { return tags.indexOf(t) < 0; });
      if (missing.length && d.source !== 'inbox') {
        row.appendChild(button('ai-accept', 'Add as tags', function () { acceptAi(d.id); }));
      }
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

  // ---- toast ----
  var toastTimer = 0;
  var undoId = '';

  function toast(msg, undo, ms) {
    $('toast-text').textContent = msg;
    undoId = undo || '';
    $('toast-undo').hidden = !undoId;
    $('toast').hidden = false;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(hideToast, ms || (undoId ? UNDO_MS : TOAST_MS));
  }

  function hideToast() {
    clearTimeout(toastTimer);
    undoId = '';
    $('toast-undo').hidden = true;
    $('toast').hidden = true;
  }

  // ---- actions (HUN-279): each IPC action asks for the board again, so counts and filters update ----
  var busy = false;

  function afterAction() {
    busy = false;
    return loadBoard();
  }

  function focusRow(id) {
    if (rowButtons[id]) rowButtons[id].focus();
  }

  function acceptAi(id) {
    if (busy) return;
    busy = true;
    var before = state.detail ? tagsOf(state.detail) : [];
    window.easyNoteIpc.acceptAi(id).then(function (d) {
      var added = tagsOf(d).filter(function (t) { return before.indexOf(t) < 0; });
      if (state.sel === id) {
        state.detail = d;
        renderDetail();
      }
      toast(added.length ? 'tags added: ' + added.map(function (t) { return '#' + t; }).join(' ') : 'no new tags');
    }, function (err) {
      toast('could not add tags: ' + err.message, '', UNDO_MS);
    }).then(afterAction);
  }

  // AC 5: the page clipboard API, no IPC. When it is missing or refused, the toast shows the id to copy by hand.
  function copyId(id) {
    function refused() { toast('copy it by hand: ' + id, '', UNDO_MS); }
    try {
      navigator.clipboard.writeText(id).then(function () { toast('Copied'); }, refused);
    } catch (e) {
      refused();
    }
  }

  function trashNote(id) {
    if (busy) return;
    busy = true;
    var shown = shownNotes().map(function (n) { return n.id; });
    var at = shown.indexOf(id);
    window.easyNoteIpc.trash(id).then(function () {
      // The row goes at once; the next row in view (as in the design) becomes the selection.
      state.notes = state.notes.filter(function (n) { return n.id !== id; });
      var rest = shown.filter(function (k) { return k !== id; });
      var next = rest.length ? rest[Math.min(Math.max(at, 0), rest.length - 1)] : '';
      state.detail = null;
      state.sel = '';
      renderDetail();
      renderList();
      if (next) {
        select(next);
        focusRow(next);
      }
      toast('moved to trash', id);
    }, function (err) {
      toast('could not move to trash: ' + err.message, '', UNDO_MS);
    }).then(afterAction);
  }

  // Undo is offered only while the toast shows it (UNDO_MS after Move to trash).
  function undoTrash() {
    var id = undoId;
    if (!id || busy) return;
    busy = true;
    hideToast();
    window.easyNoteIpc.restore(id).then(function () {
      toast('restored');
      return afterAction().then(function () {
        select(id);
        focusRow(id);
      });
    }, function (err) {
      toast('could not undo: ' + err.message, '', UNDO_MS);
      return afterAction();
    });
  }

  var loading = null;
  var loadedAt = 0;

  // ---- capture dialog (HUN-280): `+ New note`, the design's dialog, `add` like `easy-note add` ----
  var keepContext = false;
  var capOpenNo = 0;
  var saving = false;

  function setInert(on) {
    document.querySelector('.header').inert = on;
    document.querySelector('.main').inert = on;
  }

  function openCapture() {
    var box = $('cap-text');
    if (!$('cap').hidden) {
      box.focus();
      return;
    }
    capOpenNo += 1;
    var no = capOpenNo;
    box.value = '';
    dropChip();
    $('cap').hidden = false;
    setInert(true);
    box.focus();
    // The chip shows the FR-027 last context; `context` never fails, and no chip is shown when it is null.
    window.easyNoteIpc.context().then(function (r) {
      var c = r && r.context;
      if (no !== capOpenNo || $('cap').hidden || !c) return;
      var label = [str(c.repo), str(c.task)].filter(Boolean).join(' · ');
      if (!label) return;
      $('cap-chip-text').textContent = label;
      $('cap-ctx').hidden = false;
      keepContext = true;
    }, function () {});
  }

  function dropChip() {
    keepContext = false;
    $('cap-ctx').hidden = true;
    $('cap-chip-text').textContent = '';
  }

  function closeCapture() {
    capOpenNo += 1;
    $('cap').hidden = true;
    $('cap-text').value = '';
    dropChip();
    setInert(false);
    $('new-note').focus();
  }

  function capKey(e) {
    var box = $('cap-text');
    if (e.isComposing || e.keyCode === 229) return;
    if (e.key === 'Escape') {
      e.preventDefault();
      closeCapture();
      toast('cancelled · nothing saved');
      return;
    }
    if (e.target !== box) return;
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      saveCapture();
    } else if (e.key === 'Backspace' && box.value === '' && keepContext) {
      e.preventDefault();
      dropChip();
    }
  }

  // AC 4: empty or whitespace-only text sends nothing. On an error the dialog and its text stay.
  function saveCapture() {
    var text = $('cap-text').value;
    if (!text.trim()) return;
    if (saving) return;
    saving = true;
    window.easyNoteIpc.add(text, keepContext).then(function (d) {
      saving = false;
      closeCapture();
      var where = [str(d.repo), str(d.task)].filter(Boolean).join(' · ');
      toast('✓ noted' + (where ? ' · ' + where : ''));
      // The new note is at the top of today: clear search and filter so it is in view, then select it.
      clearTimeout(searchTimer);
      state.query = '';
      $('search').value = '';
      state.filter = 'all';
      return loadBoard().then(function () {
        select(d.id);
        focusRow(d.id);
      });
    }, function (err) {
      saving = false;
      toast('could not save: ' + err.message, '', UNDO_MS);
    });
  }

  function loadBoard() {
    var p = window.easyNoteIpc.board().then(function (b) {
      loadedAt = Date.now();
      state.board = b;
      state.notes = Array.isArray(b.notes) ? b.notes : [];
      state.notes.forEach(function (n) { n.hay = hayOf(n); });
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
    loading = p;
    p.then(function () { if (loading === p) loading = null; });
    return p;
  }

  // The shell's focus refresh. At window open it comes with the page's own start-up load, so it joins
  // a load in flight, and is skipped right after an answer; actions call loadBoard and always ask.
  function refreshBoard() {
    if (loading) return loading;
    if (loadedAt && Date.now() - loadedAt < FRESH_MS) return Promise.resolve();
    return loadBoard();
  }

  // `reveal` (a row click only): bring the panel into view once the detail is rendered.
  function select(id, reveal) {
    state.sel = id;
    state.detailError = '';
    Object.keys(rowButtons).forEach(function (k) { rowButtons[k].setAttribute('aria-current', k === id ? 'true' : 'false'); });
    window.easyNoteIpc.detail(id).then(function (d) {
      if (state.sel !== id) return;
      state.detail = d;
      renderDetail();
      if (reveal) revealDetail();
    }, function (err) {
      if (state.sel !== id) return;
      state.detail = null;
      state.detailError = err.message;
      renderDetail();
      if (reveal) revealDetail();
    });
  }

  $('toast-undo').addEventListener('click', undoTrash);
  $('new-note').addEventListener('click', openCapture);
  $('cap-dialog').addEventListener('keydown', capKey);
  // A click on the backdrop or the dialog's own padding would move focus to the page, where keys reach no handler.
  $('cap').addEventListener('mousedown', function (e) {
    if (e.target.closest('button, textarea')) return;
    e.preventDefault();
    $('cap-text').focus();
  });
  $('cap-drop').addEventListener('click', function () {
    dropChip();
    $('cap-text').focus();
  });
  var searchTimer = 0;
  $('search').addEventListener('input', function (e) {
    var value = e.target.value;
    clearTimeout(searchTimer);
    searchTimer = setTimeout(function () { state.query = value; renderList(); }, SEARCH_MS);
  });
  $('clear-search').addEventListener('click', function () {
    clearTimeout(searchTimer);
    state.query = '';
    $('search').value = '';
    renderList();
  });
  // The Rust shell calls window.__easyNote.refresh() on each window focus (mod.rs REFRESH).
  window.__easyNote.refresh = refreshBoard;
  buildFilters();
  renderList();
  loadBoard();
})();
