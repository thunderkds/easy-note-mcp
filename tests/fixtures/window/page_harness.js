'use strict';
// HUN-282 page harness: runs the real ipc.js and app.js in a node `vm` context over a small fake
// DOM and a stub `window.ipc` that counts requests; prints one JSON object of results.
// Usage: node page_harness.js <assets dir> <board.json>
// The board answer comes from `window::api::handle`; no network, no real notes dir.
const fs = require('fs');
const vm = require('vm');
const path = require('path');

const [assets, boardFile] = process.argv.slice(2);
const board = JSON.parse(fs.readFileSync(boardFile, 'utf8'));
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
  const ctx = vm.createContext({ document: doc, setTimeout, clearTimeout, navigator: {}, console });
  ctx.window = ctx;
  ctx.ipc = {
    postMessage(s) {
      const m = JSON.parse(s);
      posts.push(m.cmd);
      const result = m.cmd === 'board' ? board : {};
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
  return run({ ctx, doc, load, boards, advance, type, view });
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
  console.log(JSON.stringify(out));
})();
