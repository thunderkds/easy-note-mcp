'use strict';
// HUN-282 page harness: runs the real ipc.js and app.js in a node `vm` context over a small fake
// DOM and a stub `window.ipc` that counts requests; prints one JSON object of results.
// Usage: node page_harness.js <assets dir> <board.json> [<details.json>]
// The board answer comes from `window::api::handle`; no network, no real notes dir.
// With <details.json> ({ roles: { editable, inbox, other }, notes: { <id>: the `detail` answer of
// `window::api::handle` } }) the HUN-286 edit scenarios run too; the stub answers `edit` as the backend contract in ipc.js says.
// With { settings: <a `settings_get` answer> } in it, the HUN-300 Settings scenarios run.
const fs = require('fs');
const vm = require('vm');
const path = require('path');

const [assets, boardFile, detailsFile] = process.argv.slice(2);
const board = JSON.parse(fs.readFileSync(boardFile, 'utf8'));
const more = detailsFile ? JSON.parse(fs.readFileSync(detailsFile, 'utf8')) : null;
const details = more && more.notes;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const ANSWER_MS = 15;

class El {
  constructor(tag, doc) {
    this.tag = tag;
    this.doc = doc;
    this.className = '';
    this.textContent = '';
    this.children = [];
    this.listeners = {};
    this.attrs = {};
    this.hidden = false;
    this.style = {};
    this.value = '';
    this.type = '';
    this.isFragment = false;
  }
  appendChild(c) {
    if (c.isFragment) this.children.push(...c.children);
    else this.children.push(c);
    return c;
  }
  replaceChildren(...cs) {
    this.children = [];
    cs.forEach((c) => this.appendChild(c));
  }
  get firstElementChild() { return this.children[0] || new El('x', this.doc); }
  get lastChild() { return this.children[this.children.length - 1] || new El('x', this.doc); }
  get content() { return new El('template', this.doc); }
  cloneNode() { return new El('clone', this.doc); }
  setAttribute(k, v) { this.attrs[k] = String(v); }
  getAttribute(k) { return this.attrs[k]; }
  addEventListener(type, fn) { (this.listeners[type] = this.listeners[type] || []).push(fn); }
  querySelector() { return new El('x', this.doc); }
  focus() { this.doc.activeElement = this; }
  setSelectionRange() {}
  scrollIntoView() {}
  get previousElementSibling() { return new El('x', this.doc); }
  fire(type, event) { (this.listeners[type] || []).forEach((fn) => fn(event || {})); }
}

function rowTexts(root) {
  const out = [];
  (function walk(e) {
    if (e.className === 'row-first') out.push(e.textContent);
    e.children.forEach(walk);
  })(root);
  return out;
}

// Every element under `root` (depth first) for which `pred` is true.
function findAll(root, pred) {
  const out = [];
  (function walk(e) {
    if (pred(e)) out.push(e);
    e.children.forEach(walk);
  })(root);
  return out;
}

function textOf(e) {
  return e.textContent + e.children.map(textOf).join('');
}

async function scenario(run) {
  const doc = { activeElement: null, byId: {}, created: {} };
  doc.getElementById = (id) => (doc.byId[id] = doc.byId[id] || new El(id, doc));
  doc.querySelector = (selector) => doc.getElementById(selector);
  doc.createElement = (tag) => {
    doc.created[tag] = (doc.created[tag] || 0) + 1;
    return new El(tag, doc);
  };
  doc.createDocumentFragment = () => Object.assign(new El('#frag', doc), { isFragment: true });
  doc.createTextNode = (t) => Object.assign(new El('#text', doc), { textContent: t });
  const posts = [];
  const msgs = [];
  const notes = JSON.parse(JSON.stringify(details || {}));
  const editErrors = [];
  const enrichErrors = [];
  const settings = JSON.parse(JSON.stringify((more && more.settings) || {}));
  const settingsErrors = { get: [], save: [] };
  let saves = 0;
  // The `settings_save` answer: the sent values as the new `settings_get`; a confirm records the model.
  const answerSettings = (m) => {
    const queued = settingsErrors[m.cmd === 'settings_get' ? 'get' : 'save'];
    if (queued.length) return { error: queued.shift() };
    if (m.cmd === 'settings_save') {
      saves += 1;
      settings.base = 'base-' + saves;
      settings.settings = JSON.parse(JSON.stringify(m.settings));
      if (m.confirm_notice) settings.notice.confirmed_model = m.settings.model;
    }
    return JSON.parse(JSON.stringify(settings));
  };
  const live = JSON.parse(JSON.stringify(board));
  const slow = { enrichMs: ANSWER_MS };
  // The `edit` answer of the backend contract: whole new text and tags, `#` dropped, each tag once,
  // AI detail stale when the text changed. A queued error is answered first, writing nothing.
  const answerEdit = (m) => {
    if (editErrors.length) return { error: editErrors.shift() };
    const d = notes[m.id];
    if (!d) return { error: 'no such note' };
    if (d.text !== m.base || JSON.stringify(d.tags || []) !== JSON.stringify(m.base_tags)) return { error: 'the note changed outside the window' };
    const text = m.text.replace(/\n*$/, '\n');
    const tags = [];
    m.tags.forEach((t) => { t = t.replace(/^#/, ''); if (t && !tags.includes(t)) tags.push(t); });
    const changed = text !== d.text;
    d.text = text;
    d.tags = tags;
    if (changed && d.ai) d.ai.stale = true;
    return JSON.parse(JSON.stringify(d));
  };
  // The `enrich` answer: the note's `detail` with new AI detail; its board row gets the AI mark. A queued error is answered first.
  const answerEnrich = (m) => {
    if (enrichErrors.length) return { error: enrichErrors.shift() };
    const d = notes[m.id];
    if (!d) return { error: 'no such note' };
    d.ai = { title: 'AI title new', kind: 'idea', summary: 'AI summary new', tags_suggested: ['alpha', 'beta'], model: 'stub-model', at: '2026-11-19T10:00:00+07:00', stale: false };
    const row = live.notes.find((n) => n.id === m.id);
    if (row) Object.assign(row, { ai: true, kind: 'idea', stale: false });
    return JSON.parse(JSON.stringify(d));
  };
  const answer = (m) => {
    if (m.cmd === 'board') return JSON.parse(JSON.stringify(live));
    if (m.cmd === 'enrich') return answerEnrich(m);
    if (m.cmd === 'detail') return notes[m.id] ? JSON.parse(JSON.stringify(notes[m.id])) : { error: 'no such note' };
    if (m.cmd === 'edit') return answerEdit(m);
    if (m.cmd === 'settings_get' || m.cmd === 'settings_save') return answerSettings(m);
    return {};
  };
  const ctx = vm.createContext({ document: doc, setTimeout, clearTimeout, navigator: {}, console, matchMedia: () => ({ matches: true }) });
  ctx.window = ctx;
  ctx.ipc = {
    postMessage(s) {
      const m = JSON.parse(s);
      posts.push(m.cmd);
      msgs.push(m);
      // `enrich` writes when the child ends, not when it starts: its AI detail shows up in `detail` only after the delay.
      const delay = m.cmd === 'enrich' ? slow.enrichMs : ANSWER_MS;
      const now = m.cmd === 'enrich' ? null : answer(m);
      setTimeout(() => ctx.__easyNote.reply(JSON.stringify(Object.assign({ req: m.req }, now || answer(m)))), delay);
    },
  };
  vm.runInContext('var __t = 0; Date.now = function () { return 1e12 + __t; };', ctx);
  const advance = (ms) => vm.runInContext('__t += ' + ms, ctx);
  const load = () => {
    // index.html's script order: the shared capture dialog (HUN-289) before app.js.
    ['ipc.js', 'capture.js', 'settings.js', 'app.js'].forEach((f) => vm.runInContext(fs.readFileSync(path.join(assets, f), 'utf8'), ctx));
  };
  const boards = () => posts.filter((c) => c === 'board').length;
  const type = (value) => doc.byId.search.fire('input', { target: { value } });
  const view = () => ({
    rows: rowTexts(doc.byId.groups),
    none: !doc.byId['no-results'].hidden,
    noneText: doc.byId['no-results-text'].textContent,
    filters: doc.byId.filters.children.map((b) => b.lastChild.textContent),
  });
  const panel = () => doc.getElementById('detail');
  const button = (label) => findAll(panel(), (e) => e.tag === 'button' && e.textContent === label)[0];
  const box = () => findAll(panel(), (e) => e.tag === 'textarea')[0];
  const tagsBox = () => findAll(panel(), (e) => e.className === 'edit-tags')[0];
  const key = (target, k, extra) => target.fire('keydown', Object.assign({ key: k, preventDefault() {} }, extra || {}));
  const edit = (text, tags) => {
    const b = box();
    b.value = text;
    b.fire('input', { target: b });
    if (tags !== undefined) {
      const t = tagsBox();
      t.value = tags;
      t.fire('input', { target: t });
    }
  };
  // The row button of note `id`, with no search or filter: rows are in day order, board order within a day.
  const rowButton = (id) => {
    const order = ['today', 'yesterday', 'earlier'].flatMap((d) => board.notes.filter((n) => n.day === d).map((n) => n.id));
    return findAll(doc.getElementById('groups'), (e) => e.className === 'row')[order.indexOf(id)];
  };
  const shown = () => ({
    editing: !!box(),
    draft: box() ? box().value : null,
    tags: tagsBox() ? tagsBox().value : null,
    words: findAll(panel(), (e) => e.className === 'words').map((e) => e.textContent),
    error: findAll(panel(), (e) => e.className === 'error').map((e) => e.textContent).join(''),
    stale: findAll(panel(), (e) => e.className === 'ai-stale').length > 0,
    buttons: findAll(panel(), (e) => e.tag === 'button').map((e) => e.textContent),
    disabled: findAll(panel(), (e) => e.tag === 'button' && e.disabled).map((e) => e.textContent),
    aiTitle: findAll(panel(), (e) => e.className === 'ai-title').map((e) => e.textContent).join(''),
    aiKind: findAll(panel(), (e) => e.className === 'ai-kind').map((e) => e.textContent).join(''),
    aiSummary: findAll(panel(), (e) => e.className === 'ai-summary').map((e) => e.textContent).join(''),
    aiTags: findAll(panel(), (e) => e.className === 'ai-tag').map((e) => e.textContent),
    readOnly: findAll(panel(), (e) => e.className === 'read-only').map((e) => e.textContent).join(''),
    id: findAll(panel(), (e) => e.className === 'card-id').map((e) => e.textContent).join(''),
    text: textOf(panel()),
    toast: doc.getElementById('toast-text').textContent,
    focusIsBox: !!box() && doc.activeElement === box(),
  });
  const rowMark = (id) => findAll(rowButton(id), (e) => String(e.className).startsWith('row-mark'))[0].textContent;
  const sent = (cmd) => msgs.filter((m) => m.cmd === cmd);
  // A scenario that throws reports { crash } instead of stopping the others.
  try {
    return await run({ ctx, doc, load, boards, advance, type, view, posts, button, box, key, edit, rowButton, rowMark, shown, sent, editErrors, enrichErrors, slow, notes, settings, settingsErrors });
  } catch (e) {
    return { crash: String(e && e.stack || e) };
  }
}

// The HUN-286 scenarios: Edit mode in the detail panel and the initial note of `dashboard --note`.
async function editScenarios(out) {
  const { editable, inbox, other } = more.roles;
  const open = async (s, id) => {
    s.ctx.__easyNote = { initialNote: id };
    s.load();
    await sleep(80);
  };
  out.ids = { editable, inbox, other };
  out.initial_note = await scenario(async (s) => {
    await open(s, editable);
    const row = s.rowButton(editable);
    return { posts: s.posts.slice(), detail: s.sent('detail').map((m) => m.id), shown: s.shown(), current: row.attrs['aria-current'] };
  });
  // HUN-290: a second `dashboard --note <id>` call; the shell asks the open page to show a note.
  out.show_request = await scenario(async (s) => {
    s.load();
    await sleep(80);
    s.ctx.__easyNote.show(editable);
    await sleep(80);
    const row = s.rowButton(editable);
    return { detail: s.sent('detail').map((m) => m.id), shown: s.shown(), current: row.attrs['aria-current'] };
  });
  out.initial_unknown = await scenario(async (s) => {
    await open(s, '20261119-090000-ffff');
    return { shown: s.shown() };
  });
  out.initial_none = await scenario(async (s) => {
    s.load();
    await sleep(80);
    return { posts: s.posts.slice(), shown: s.shown() };
  });
  out.edit_open = await scenario(async (s) => {
    await open(s, editable);
    s.button('Edit').fire('click');
    return { shown: s.shown() };
  });
  out.edit_save = await scenario(async (s) => {
    await open(s, editable);
    const boards = s.boards();
    s.button('Edit').fire('click');
    s.edit('New text #Web\nsecond line', '#y, new');
    s.key(s.box(), 'Enter', { ctrlKey: true });
    await sleep(80);
    return { sent: s.sent('edit'), base: details[editable].text, shown: s.shown(), boards: s.boards() - boards };
  });
  out.edit_save_button = await scenario(async (s) => {
    await open(s, editable);
    s.button('Edit').fire('click');
    s.edit('by the button');
    s.button('Save').fire('click');
    await sleep(80);
    return { sent: s.sent('edit').length, shown: s.shown() };
  });
  out.edit_cancel = await scenario(async (s) => {
    await open(s, editable);
    s.button('Edit').fire('click');
    s.edit('a draft to drop', '#gone');
    s.key(s.box(), 'Escape');
    await sleep(40);
    const esc = { sent: s.sent('edit').length, shown: s.shown() };
    s.button('Edit').fire('click');
    s.edit('another draft');
    s.button('Cancel').fire('click');
    await sleep(40);
    return { esc, button: { sent: s.sent('edit').length, shown: s.shown() } };
  });
  out.edit_composing = await scenario(async (s) => {
    await open(s, editable);
    s.button('Edit').fire('click');
    s.edit('typing with an IME');
    s.key(s.box(), 'Enter', { ctrlKey: true, isComposing: true });
    s.key(s.box(), 'Escape', { isComposing: true });
    s.key(s.box(), 'Enter', { ctrlKey: true, keyCode: 229 });
    await sleep(40);
    return { sent: s.sent('edit').length, shown: s.shown() };
  });
  out.edit_refresh = await scenario(async (s) => {
    await open(s, editable);
    s.button('Edit').fire('click');
    s.edit('a draft kept over a refresh', '#kept');
    const theBox = s.box();
    const boards = s.boards();
    s.advance(60000);
    s.ctx.__easyNote.refresh();
    await sleep(80);
    return { boards: s.boards() - boards, sameBox: s.box() === theBox, shown: s.shown(), sent: s.sent('edit').length };
  });
  out.edit_changed_outside = await scenario(async (s) => {
    await open(s, editable);
    s.button('Edit').fire('click');
    s.edit('my draft');
    // Another app writes the file after the page loaded it.
    s.notes[editable].text = 'changed in Obsidian\n';
    s.notes[editable].tags = ['y', 'obsidian'];
    s.key(s.box(), 'Enter', { ctrlKey: true });
    await sleep(80);
    const refused = { shown: s.shown(), detailsAfter: s.sent('detail').length };
    s.key(s.box(), 'Enter', { ctrlKey: true });
    await sleep(80);
    return { refused, second: s.sent('edit')[1], shown: s.shown() };
  });
  out.edit_error = await scenario(async (s) => {
    await open(s, editable);
    s.button('Edit').fire('click');
    s.edit('x'.repeat(10));
    s.editErrors.push('the text is too long');
    s.key(s.box(), 'Enter', { ctrlKey: true });
    await sleep(80);
    return { shown: s.shown() };
  });
  out.edit_empty = await scenario(async (s) => {
    await open(s, editable);
    s.button('Edit').fire('click');
    s.edit('  \n\t ');
    s.key(s.box(), 'Enter', { ctrlKey: true });
    await sleep(40);
    return { sent: s.sent('edit').length, shown: s.shown() };
  });
  out.edit_switch = await scenario(async (s) => {
    await open(s, editable);
    s.button('Edit').fire('click');
    s.edit('a draft I want to keep');
    const asked = s.sent('detail').length;
    s.rowButton(other).fire('click');
    await sleep(80);
    const blocked = { shown: s.shown(), detail: s.sent('detail').length - asked, current: s.rowButton(other).attrs['aria-current'] };
    s.key(s.box(), 'Escape');
    s.rowButton(other).fire('click');
    await sleep(80);
    return { blocked, after: s.shown() };
  });
  out.inbox = await scenario(async (s) => {
    await open(s, inbox);
    return { shown: s.shown() };
  });
}

// The HUN-288 scenarios: `Run AI` in the detail panel.
async function runAiScenarios(out) {
  const { editable, inbox, other, plain } = more.roles;
  const open = async (s, id) => {
    s.ctx.__easyNote = { initialNote: id };
    s.load();
    await sleep(80);
  };
  out.ai_idle = await scenario(async (s) => {
    await open(s, plain);
    const none = s.shown();
    return { none };
  });
  out.ai_idle_with_ai = await scenario(async (s) => {
    await open(s, editable);
    return { shown: s.shown() };
  });
  out.ai_inbox = await scenario(async (s) => {
    await open(s, inbox);
    return { shown: s.shown() };
  });
  out.ai_run = await scenario(async (s) => {
    await open(s, plain);
    const boards = s.boards();
    const markBefore = s.rowMark(plain);
    s.button('Run AI').fire('click');
    const running = s.shown();
    s.button('AI running…').fire('click');
    s.button('AI running…').fire('click');
    await sleep(80);
    return { markBefore, running, sent: s.sent('enrich'), shown: s.shown(), boards: s.boards() - boards, markAfter: s.rowMark(plain) };
  });
  out.ai_again = await scenario(async (s) => {
    await open(s, editable);
    s.button('Re-run').fire('click');
    const running = s.shown();
    await sleep(80);
    return { running, shown: s.shown() };
  });
  out.ai_error = await scenario(async (s) => {
    await open(s, plain);
    s.enrichErrors.push('claude not found (claude): install it, or set [ai] command in x');
    s.button('Run AI').fire('click');
    await sleep(80);
    const first = s.shown();
    s.button('Run AI').fire('click');
    await sleep(80);
    return { first, second: s.shown(), sent: s.sent('enrich').length };
  });
  out.ai_switch = await scenario(async (s) => {
    await open(s, plain);
    s.slow.enrichMs = 250;
    s.button('Run AI').fire('click');
    await sleep(20);
    s.rowButton(other).fire('click');
    await sleep(60);
    const away = s.shown();
    s.rowButton(plain).fire('click');
    await sleep(60);
    const back = s.shown();
    await sleep(300);
    const done = s.shown();
    s.rowButton(other).fire('click');
    await sleep(60);
    const elsewhere = s.shown();
    return { away, back, done, elsewhere, sent: s.sent('enrich').length, mark: s.rowMark(plain) };
  });
  out.ai_answer_elsewhere = await scenario(async (s) => {
    await open(s, plain);
    s.slow.enrichMs = 150;
    s.button('Run AI').fire('click');
    await sleep(20);
    s.rowButton(other).fire('click');
    await sleep(300);
    const shownOther = s.shown();
    s.rowButton(plain).fire('click');
    await sleep(60);
    return { shownOther, mark: s.rowMark(plain), back: s.shown() };
  });
  out.ai_while_editing = await scenario(async (s) => {
    await open(s, plain);
    s.slow.enrichMs = 100;
    s.button('Run AI').fire('click');
    await sleep(10);
    s.button('Edit').fire('click');
    s.edit('my draft kept');
    await sleep(200);
    const editing = s.shown();
    s.button('Cancel').fire('click');
    await sleep(20);
    return { editing, after: s.shown() };
  });
}

// The HUN-300 scenarios: the Settings dialog and the "what is sent" notice.
async function settingsScenarios(out) {
  const ui = (s) => {
    const at = (id) => s.doc.getElementById(id);
    const click = (id) => at(id).fire('click', { target: at(id) });
    const type = (id, value) => { at(id).value = value; at(id).fire('input', { target: at(id) }); };
    const press = (id, k) => at(id).fire('keydown', { key: k, target: at(id), preventDefault() {}, stopPropagation() {} });
    const state = () => ({
      open: !at('set').hidden,
      asking: !at('ask').hidden,
      ask: at('ask-text').textContent,
      loading: !at('set-loading').hidden,
      form: !at('set-form').hidden,
      save: !at('set-save').hidden,
      retry: !at('set-retry').hidden,
      enrich: at('set-enrich').getAttribute('aria-checked'),
      schedule: at('set-schedule').getAttribute('aria-checked'),
      scheduleDisabled: !!at('set-schedule').disabled,
      reason: !at('set-schedule-reason').hidden,
      values: [at('set-interval').value, at('set-max').value, at('set-batch').value, at('set-model').value],
      jobs: ['organize', 'drafts', 'digest'].map((j) => !!at('set-job-' + j).checked),
      afterSave: !!at('set-after-save').checked,
      readOnly: [at('set-path').textContent, at('set-command').textContent, at('set-account').textContent],
      error: at('set-error').hidden ? '' : at('set-error').textContent,
      toast: at('toast').hidden ? '' : at('toast-text').textContent,
      focus: s.doc.activeElement ? s.doc.activeElement.tag : '',
      inert: !!(at('set').inert),
    });
    const saves = () => s.sent('settings_save');
    // Load the page and open Settings from the gear; `change` edits the file's answer first.
    const open = async (change) => {
      if (change) change(s.settings);
      s.load();
      await sleep(60);
      click('settings-open');
      await sleep(60);
    };
    return { at, click, type, press, state, saves, open };
  };
  // index.html starts these hidden.
  const hidden = ['set', 'ask', 'toast', 'set-loading', 'set-form', 'set-save', 'set-retry', 'set-error'];
  const run = (name, body) => scenario((s) => {
    hidden.forEach((id) => { s.doc.getElementById(id).hidden = true; });
    return body(ui(s), s);
  }).then((r) => { out[name] = r; });

  await run('set_open', async (u, s) => {
    s.load();
    await sleep(60);
    u.click('settings-open');
    const loading = u.state();
    await sleep(60);
    return { loading, shown: u.state(), posts: s.posts.slice() };
  });
  await run('set_disabled', async (u) => {
    await u.open();
    u.click('set-schedule');
    await sleep(40);
    return { shown: u.state(), saves: u.saves().length };
  });
  await run('set_cancel', async (u, s) => {
    await u.open();
    u.click('set-enrich');
    u.click('set-schedule');
    const asked = u.state();
    u.click('ask-cancel');
    await sleep(40);
    const cancelled = u.state();
    u.click('set-schedule');
    u.press('ask-dialog', 'Escape');
    await sleep(40);
    return { asked, cancelled, escaped: u.state(), saves: u.saves().length, posts: s.posts.slice() };
  });
  await run('set_confirm', async (u, s) => {
    await u.open();
    u.click('set-enrich');
    u.click('set-schedule');
    u.click('ask-confirm');
    u.click('ask-confirm');
    await sleep(60);
    return { shown: u.state(), saves: u.saves(), posts: s.posts.slice() };
  });
  await run('set_seen', async (u) => {
    await u.open((a) => { a.settings.enrich = 'claude'; a.notice.confirmed_model = 'haiku'; });
    u.click('set-schedule');
    const flipped = { shown: u.state(), saves: u.saves().length };
    u.click('set-save');
    u.click('set-save');
    await sleep(60);
    return { flipped, shown: u.state(), saves: u.saves() };
  });
  await run('set_model_changed', async (u) => {
    await u.open((a) => { a.settings.enrich = 'claude'; a.notice.confirmed_model = 'haiku'; });
    u.click('set-schedule');
    u.type('set-model', ' sonnet ');
    u.click('set-save');
    const asked = { shown: u.state(), saves: u.saves().length };
    u.click('ask-confirm');
    await sleep(60);
    return { asked, shown: u.state(), saves: u.saves() };
  });
  await run('set_already_on', async (u) => {
    await u.open((a) => { a.settings.enrich = 'claude'; a.settings.schedule = 'tray'; });
    const first = u.state();
    u.type('set-model', 'sonnet');
    u.click('set-save');
    await sleep(60);
    return { first, shown: u.state(), saves: u.saves() };
  });
  await run('set_values', async (u) => {
    await u.open();
    u.type('set-interval', ' 30 ');
    u.type('set-max', '40');
    u.type('set-batch', '8');
    u.at('set-job-drafts').checked = false;
    u.at('set-after-save').checked = true;
    u.press('set-interval', 'Enter');
    await sleep(60);
    return { shown: u.state(), saves: u.saves() };
  });
  await run('set_off_while_on', async (u) => {
    await u.open((a) => { a.settings.enrich = 'claude'; a.settings.schedule = 'tray'; });
    u.click('set-enrich');
    const off = u.state();
    u.click('set-save');
    await sleep(60);
    return { off, saves: u.saves() };
  });
  await run('set_error', async (u, s) => {
    await u.open();
    u.type('set-interval', 'abc');
    s.settingsErrors.save.push('/x/config.toml: [ai] interval_min must be a whole number from 5 to 9');
    u.click('set-save');
    await sleep(60);
    return { shown: u.state(), saves: u.saves() };
  });
  await run('set_confirm_error', async (u, s) => {
    await u.open();
    u.click('set-enrich');
    u.click('set-schedule');
    s.settingsErrors.save.push('cannot write /x/config.toml: Permission denied (os error 13)');
    u.click('ask-confirm');
    await sleep(60);
    return { shown: u.state(), saves: u.saves().length };
  });
  await run('set_outside', async (u, s) => {
    await u.open();
    u.type('set-interval', '30');
    s.settingsErrors.save.push('the settings changed outside the window');
    s.settings.base = 'newer';
    s.settings.settings.interval_min = 15;
    u.click('set-save');
    await sleep(80);
    const refused = u.state();
    u.click('set-save');
    await sleep(60);
    return { refused, gets: s.sent('settings_get').length, saves: u.saves() };
  });
  await run('set_unreadable', async (u, s) => {
    s.settingsErrors.get.push('cannot read /x/config.toml: TOML parse error at line 1, column 4');
    await u.open();
    const failed = u.state();
    u.click('set-retry');
    await sleep(60);
    return { failed, retried: u.state(), gets: s.sent('settings_get').length };
  });
  await run('set_close', async (u, s) => {
    await u.open();
    u.type('set-interval', '30');
    u.press('set-dialog', 'Escape');
    await sleep(20);
    const escaped = u.state();
    u.click('settings-open');
    await sleep(60);
    const reopened = u.state();
    u.click('set-close');
    await sleep(20);
    return { escaped, reopened, closed: u.state(), gets: s.sent('settings_get').length, saves: u.saves().length };
  });
}

(async () => {
  const out = {};
  out.focus_before_answer = await scenario(async (s) => {
    s.load();
    s.ctx.__easyNote.refresh();
    await sleep(80);
    return { boards: s.boards() };
  });
  out.focus_after_answer = await scenario(async (s) => {
    s.load();
    await sleep(80);
    s.advance(200);
    s.ctx.__easyNote.refresh();
    await sleep(80);
    return { boards: s.boards() };
  });
  out.refresh_much_later = await scenario(async (s) => {
    s.load();
    await sleep(80);
    s.advance(60000);
    s.ctx.__easyNote.refresh();
    await sleep(80);
    return { boards: s.boards() };
  });
  out.search_typing = await scenario(async (s) => {
    s.load();
    await sleep(80);
    const before = s.doc.created.li || 0;
    const all = s.view();
    for (const q of ['f', 'fi', 'fir', 'fire', 'firef', 'firefo', 'firefox', 'firefox ', 'firefox f', 'firefox flaky']) s.type(q);
    await sleep(300);
    const typed = s.view();
    const rebuilt = (s.doc.created.li || 0) - before;
    s.doc.byId.filters.children[1].fire('click');
    await sleep(300);
    const bugs = s.view();
    s.doc.byId.filters.children[0].fire('click');
    s.type('');
    await sleep(300);
    return { all, typed, rebuilt, bugs, cleared: s.view() };
  });
  out.search_body_word = await scenario(async (s) => {
    s.load();
    await sleep(80);
    s.type('UI_WAIT_FOR');
    await sleep(300);
    const body = s.view();
    s.type('zzzz-nothing');
    await sleep(300);
    return { body, none: s.view() };
  });
  if (details) await editScenarios(out);
  if (details) await runAiScenarios(out);
  if (more && more.settings) await settingsScenarios(out);
  console.log(JSON.stringify(out));
})();
