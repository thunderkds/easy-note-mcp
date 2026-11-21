'use strict';
// HUN-282 page harness: runs the real ipc.js and app.js in a node `vm` context over a small fake
// DOM and a stub `window.ipc` that counts requests; prints one JSON object of results.
// Usage: node page_harness.js <assets dir> <board.json> [<details.json>]
// The board answer comes from `window::api::handle`; no network, no real notes dir.
// With <details.json> ({ roles: { editable, inbox, other }, notes: { <id>: the `detail` answer of
// `window::api::handle` } }) the HUN-286 edit scenarios run too; the stub answers `edit` as the backend contract in ipc.js says.
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
  // The `edit` answer of the backend contract: whole new text and tags, `#` dropped, each tag once,
  // AI detail stale when the text changed. A queued error is answered first, writing nothing.
  const answerEdit = (m) => {
    if (editErrors.length) return { error: editErrors.shift() };
    const d = notes[m.id];
    if (!d) return { error: 'no such note' };
    if (d.text !== m.base) return { error: 'the note changed outside the window' };
    const text = m.text.replace(/\n*$/, '\n');
    const tags = [];
    m.tags.forEach((t) => { t = t.replace(/^#/, ''); if (t && !tags.includes(t)) tags.push(t); });
    const changed = text !== d.text;
    d.text = text;
    d.tags = tags;
    if (changed && d.ai) d.ai.stale = true;
    return JSON.parse(JSON.stringify(d));
  };
  const answer = (m) => {
    if (m.cmd === 'board') return board;
    if (m.cmd === 'detail') return notes[m.id] ? JSON.parse(JSON.stringify(notes[m.id])) : { error: 'no such note' };
    if (m.cmd === 'edit') return answerEdit(m);
    return {};
  };
  const ctx = vm.createContext({ document: doc, setTimeout, clearTimeout, navigator: {}, console, matchMedia: () => ({ matches: true }) });
  ctx.window = ctx;
  ctx.ipc = {
    postMessage(s) {
      const m = JSON.parse(s);
      posts.push(m.cmd);
      msgs.push(m);
      const result = answer(m);
      setTimeout(() => ctx.__easyNote.reply(JSON.stringify(Object.assign({ req: m.req }, result))), ANSWER_MS);
    },
  };
  vm.runInContext('var __t = 0; Date.now = function () { return 1e12 + __t; };', ctx);
  const advance = (ms) => vm.runInContext('__t += ' + ms, ctx);
  const load = () => {
    vm.runInContext(fs.readFileSync(path.join(assets, 'ipc.js'), 'utf8'), ctx);
    vm.runInContext(fs.readFileSync(path.join(assets, 'app.js'), 'utf8'), ctx);
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
    readOnly: findAll(panel(), (e) => e.className === 'read-only').map((e) => e.textContent).join(''),
    id: findAll(panel(), (e) => e.className === 'card-id').map((e) => e.textContent).join(''),
    text: textOf(panel()),
    toast: doc.getElementById('toast-text').textContent,
    focusIsBox: !!box() && doc.activeElement === box(),
  });
  const sent = (cmd) => msgs.filter((m) => m.cmd === cmd);
  // A scenario that throws reports { crash } instead of stopping the others.
  try {
    return await run({ ctx, doc, load, boards, advance, type, view, posts, button, box, key, edit, rowButton, shown, sent, editErrors, notes });
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
  console.log(JSON.stringify(out));
})();
