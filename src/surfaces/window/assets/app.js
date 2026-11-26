/* easy-note dashboard page (HUN-278 read, HUN-279 actions, HUN-280 capture (the dialog is capture.js), HUN-286 edit, HUN-288 Run AI). Port of docs/design/Main.dc.html, board "Dashboard (clickable)".
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

  // `edit` (HUN-286) is the draft while the panel is in edit mode: { id, base, baseTags, text, tags, text0, tags0, error, outside }.
  // It lives here, not only in the DOM, so a re-render keeps it; the focus refresh never re-renders the panel.
  var state = { board: null, notes: [], query: '', filter: 'all', sel: '', detail: null, detailError: '', edit: null };
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

  // `edited <when>`, or nothing for a note never edited.
  function editedText(when) { return when ? 'edited ' + str(when) : ''; }

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
    // `edited` (HUN-291) leads the line, beside the time, so a narrow row cuts the project and tags, not it.
    body.appendChild(el('span', 'row-where', [editedText(n.edited), str(n.repo) || 'no project', str(n.task), tags].filter(Boolean).join(' · ')));
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
    if (state.edit && state.edit.id === d.id) {
      aside.replaceChildren(renderEdit(d, state.edit));
      return;
    }
    var card = el('article', 'card');
    card.appendChild(cardHead(d));

    card.appendChild(el('p', 'words-label', 'Your words'));
    card.appendChild(el('p', 'words', d.text));
    if (d.masked) card.appendChild(el('p', 'masked', 'secret masked'));

    var ctx = d.context;
    var branch = typeof ctx === 'string' ? ctx : ctx && ctx.branch;
    var when = d.day === 'today' ? 'today ' + str(d.time) : d.day === 'yesterday' ? 'yesterday ' + str(d.time) : rowTime(d);
    var context = el('p', 'context', [str(d.repo) || 'no project', str(branch), str(d.task), when].filter(Boolean).join(' · '));
    // The full stamp (HUN-291) is one span, so a narrow panel wraps before it and never inside the date.
    if (d.edited_at) {
      context.appendChild(document.createTextNode(' · '));
      context.appendChild(el('span', 'edited-at', editedText(d.edited_at)));
    }
    card.appendChild(context);

    var tagBox = el('div', 'tags');
    var tags = tagsOf(d);
    tags.forEach(function (t) { tagBox.appendChild(el('span', 'tag', '#' + t)); });
    if (!tags.length) tagBox.appendChild(el('span', 'no-tags', 'no tags'));
    card.appendChild(tagBox);

    if (d.ai) card.appendChild(renderAi(d.ai, d));
    else {
      var none = el('p', 'no-ai', 'No AI detail yet. Press Run AI, or in Claude Code run the ');
      none.appendChild(el('code', null, 'enrich_notes'));
      none.appendChild(document.createTextNode(' prompt; your own AI writes it in a separate file.'));
      card.appendChild(none);
    }

    var acts = el('div', 'actions');
    // Inbox notes are read-only (FR-013): no Edit button, a line that says so.
    editButton = null;
    if (d.source === 'inbox') acts.appendChild(el('span', 'read-only', 'Inbox notes are read-only.'));
    else {
      editButton = button('act', 'Edit', function () { startEdit(d); });
      acts.appendChild(editButton);
    }
    var running = !!aiRunning[d.id];
    var runAi = button('act', running ? 'AI running…' : d.ai ? 'Re-run' : 'Run AI', function () { runAiFor(d.id); });
    runAi.disabled = running;
    acts.appendChild(runAi);
    acts.appendChild(button('act', 'Copy id', function () { copyId(d.id); }));
    acts.appendChild(button('act danger', 'Move to trash', function () { trashNote(d.id); }));
    card.appendChild(acts);
    aside.replaceChildren(card);
  }

  function cardHead(d) {
    var head = el('div', 'card-head');
    head.appendChild(el('span', 'card-id', d.id));
    head.appendChild(el('span', 'chip', d.source));
    return head;
  }

  // ---- edit mode (HUN-286): the note's text and tags, saved by `edit` like `easy-note edit` ----
  var OUTSIDE = 'the note changed outside the window';
  var editButton = null;
  var editBox = null;

  function renderEdit(d, e) {
    var card = el('article', 'card');
    card.appendChild(cardHead(d));
    if (e.error) {
      var err = el('p', 'error', e.error);
      err.setAttribute('role', 'alert');
      card.appendChild(err);
    }
    // After a save refused as changed outside the window: the text in the file now, above the kept draft.
    if (e.outside) {
      card.appendChild(el('p', 'words-label', 'Text in the file now'));
      card.appendChild(el('p', 'words', d.text));
    }
    var label = el('label', 'edit-label', e.outside ? 'Your draft' : 'Your words');
    label.htmlFor = 'edit-text';
    card.appendChild(label);
    var box = el('textarea', 'edit-text');
    box.id = 'edit-text';
    box.rows = 8;
    box.value = e.text;
    box.addEventListener('input', function () { e.text = box.value; });
    box.addEventListener('keydown', editKey);
    card.appendChild(box);
    editBox = box;

    var tagLabel = el('label', 'edit-label', 'Tags');
    tagLabel.htmlFor = 'edit-tags';
    card.appendChild(tagLabel);
    var tags = el('input', 'edit-tags');
    tags.id = 'edit-tags';
    tags.type = 'text';
    tags.value = e.tags;
    tags.placeholder = '#tag #other';
    tags.setAttribute('autocomplete', 'off');
    tags.addEventListener('input', function () { e.tags = tags.value; });
    tags.addEventListener('keydown', editKey);
    card.appendChild(tags);

    var keys = el('div', 'edit-keys');
    [['Ctrl+Enter', ' save'], ['Esc', ' cancel']].forEach(function (k) {
      var s = el('span');
      s.appendChild(el('kbd', null, k[0]));
      s.appendChild(document.createTextNode(k[1]));
      keys.appendChild(s);
    });
    card.appendChild(keys);

    var acts = el('div', 'actions');
    acts.appendChild(button('act primary', 'Save', saveEdit));
    acts.appendChild(button('act', 'Cancel', cancelEdit));
    card.appendChild(acts);
    return card;
  }

  function focusEditBox() {
    if (!editBox) return;
    editBox.focus();
    editBox.setSelectionRange(editBox.value.length, editBox.value.length);
  }

  function startEdit(d) {
    if (busy || state.sel !== d.id) return;
    var text = str(d.text).replace(/\n$/, '');
    var tags = tagsOf(d).map(function (t) { return '#' + t; }).join(' ');
    state.edit = { id: d.id, base: str(d.text), baseTags: tagsOf(d), text: text, tags: tags, text0: text, tags0: tags, error: '', outside: false };
    renderDetail();
    focusEditBox();
  }

  function leaveEdit() {
    state.edit = null;
    editBox = null;
    renderDetail();
    if (editButton) editButton.focus();
  }

  function cancelEdit() {
    if (!state.edit || busy) return;
    leaveEdit();
    toast('cancelled · nothing saved');
  }

  // IME composition: no key does anything while a word is being composed.
  function editKey(e) {
    if (e.isComposing || e.keyCode === 229) return;
    if (e.key === 'Escape') {
      e.preventDefault();
      cancelEdit();
    } else if (e.key === 'Enter' && e.ctrlKey) {
      e.preventDefault();
      saveEdit();
    }
  }

  function editFailed(e, msg) {
    e.error = msg;
    if (state.edit !== e) return;
    renderDetail();
    focusEditBox();
  }

  // AC 4: one `edit` request; the board is asked again after it (the list row). AC 2: a save refused
  // as changed outside the window asks for the note again and keeps the draft; the next save uses the
  // reloaded text as its base.
  function saveEdit() {
    var e = state.edit;
    if (!e || busy) return;
    if (!e.text.trim()) {
      editFailed(e, 'The text is empty. Write the note, or press Esc to cancel.');
      return;
    }
    if (e.text === e.text0 && e.tags === e.tags0 && !e.outside) {
      leaveEdit();
      toast('no change · nothing saved');
      return;
    }
    busy = true;
    var id = e.id;
    var tags = e.tags.split(/[\s,]+/).filter(Boolean);
    window.easyNoteIpc.edit(id, e.text, tags, e.base, e.baseTags).then(function (d) {
      if (state.edit === e) state.edit = null;
      editBox = null;
      if (state.sel === id) {
        state.detail = d;
        renderDetail();
        if (editButton) editButton.focus();
      }
      toast('saved');
    }, function (err) {
      if (err.message !== OUTSIDE) {
        editFailed(e, 'could not save: ' + err.message);
        return;
      }
      return window.easyNoteIpc.detail(id).then(function (d) {
        e.base = str(d.text);
        e.baseTags = tagsOf(d);
        e.outside = true;
        if (state.sel === id) state.detail = d;
        editFailed(e, 'Not saved: ' + OUTSIDE + '. Your draft is kept below the text in the file now. Save again to replace that text.');
      }, function (err2) {
        editFailed(e, 'Not saved: ' + OUTSIDE + '. Could not read the note again: ' + err2.message);
      });
    }).then(afterAction);
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

  // HUN-288: `Run AI` starts the person's `claude` for one note and the answer comes seconds later, so it
  // does not use `busy` (the other actions stay usable) and keeps its own state per note id: one run per
  // note at a time, whichever note is selected when the answer comes.
  var aiRunning = {};

  function runAiFor(id) {
    if (aiRunning[id]) return;
    aiRunning[id] = true;
    if (state.sel === id) renderDetail();
    window.easyNoteIpc.enrich(id).then(function (d) {
      delete aiRunning[id];
      // Another note is selected now: its panel stays; coming back asks `detail`, which holds the new AI detail.
      if (state.sel !== id) return;
      state.detail = d;
      // In edit mode the draft stays; Cancel and Save show the new AI detail.
      if (!state.edit) renderDetail();
    }, function (err) {
      delete aiRunning[id];
      if (state.sel === id && !state.edit) renderDetail();
      toast('Run AI failed: ' + err.message, '', UNDO_MS);
    }).then(loadBoard);
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

  // ---- capture dialog (HUN-280): `+ New note`; the dialog itself is capture.js, shared with `quick` (HUN-289) ----
  function setInert(on) {
    document.querySelector('.header').inert = on;
    document.querySelector('.main').inert = on;
  }

  // AC 5: close, saved toast, board reload, select the new note. An error keeps the dialog and its text.
  var capture = window.easyNoteCapture.attach({
    onOpen: function () { setInert(true); },
    onClose: function () {
      setInert(false);
      $('new-note').focus();
    },
    onCancel: function () { toast('cancelled · nothing saved'); },
    onError: function (message) { toast('could not save: ' + message, '', UNDO_MS); },
    onSaved: function (d) {
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
    }
  });

  // ---- Settings (HUN-300): the gear and its dialogs are settings.js ----
  window.easyNoteSettings.attach({
    onOpen: function () { setInert(true); },
    onClose: function () {
      setInert(false);
      $('settings-open').focus();
    },
    toast: function (message, ms) { toast(message, '', ms); }
  });

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
        state.edit = null;
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

  // `reveal` (a row click, or the `--note` initial note): bring the panel into view once the detail is rendered.
  // HUN-286 (Questions Log): while a note is in edit mode the selection stays on it. A row click
  // says so and puts the caret back in the draft; other callers (Undo, a new note) change nothing.
  function select(id, reveal) {
    if (state.edit) {
      if (reveal && id !== state.edit.id) toast('save or cancel the edit first');
      if (reveal) focusEditBox();
      return;
    }
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
  $('new-note').addEventListener('click', capture.open);
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
  // A second `easy-note dashboard --note <id>` (HUN-290): the shell asks the open window to show a note.
  window.__easyNote.show = function (id) { select(id, true); focusRow(id); };
  buildFilters();
  renderList();
  // `easy-note dashboard --note <id>` (HUN-286): select that note once the first board answer is in,
  // and bring the panel into view as a row click does (it is below the list in a narrow window).
  loadBoard().then(function () {
    var first = window.__easyNote.initialNote;
    if (!first) return;
    select(first, true);
    focusRow(first);
  });
})();
