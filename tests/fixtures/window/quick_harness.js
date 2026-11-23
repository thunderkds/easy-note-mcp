'use strict';
// HUN-289 quick page harness: runs the real ipc.js, capture.js and quick.js in a node `vm` context
// over a small fake DOM and a stub `window.ipc`; prints one JSON object of results.
// Usage: node quick_harness.js <assets dir>
// The stub answers `context` with a chip and gives no reply to `add` and `close`, as the quick window does;
// with `addError` it answers `add` with that error, as `capture_ipc` does for too long a text.
const fs = require('fs');
const vm = require('vm');
const path = require('path');

const [assets] = process.argv.slice(2);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

class El {
  constructor(id, doc) { this.id = id; this.doc = doc; this.hidden = false; this.value = ''; this.textContent = ''; this.listeners = {}; }
  addEventListener(type, fn) { (this.listeners[type] = this.listeners[type] || []).push(fn); }
  focus() { this.doc.activeElement = this; }
  closest() { return null; }
  fire(type, event) { (this.listeners[type] || []).forEach((fn) => fn(event || {})); }
}

async function scenario(chip, run, early, addError) {
  const doc = { activeElement: null, byId: {} };
  doc.getElementById = (id) => (doc.byId[id] = doc.byId[id] || new El(id, doc));
  // quick.html: the dialog and the chip row start hidden.
  doc.getElementById('cap').hidden = true;
  doc.getElementById('cap-ctx').hidden = true;
  doc.getElementById('cap-error').hidden = true;
  const msgs = [];
  const ctx = vm.createContext({ document: doc, setTimeout, clearTimeout, console });
  ctx.window = ctx;
  ctx.ipc = {
    postMessage(s) {
      const m = JSON.parse(s);
      msgs.push(m);
      if (m.cmd === 'context') setTimeout(() => ctx.__easyNote.reply(JSON.stringify({ req: m.req, context: chip })), 5);
      if (m.cmd === 'add' && addError) setTimeout(() => ctx.__easyNote.reply(JSON.stringify({ req: m.req, error: addError })), 5);
    },
  };
  ['ipc.js', 'capture.js', 'quick.js'].forEach((f) => vm.runInContext(fs.readFileSync(path.join(assets, f), 'utf8'), ctx));
  // `early`: the keys come before the `context` reply (a fast Enter).
  if (!early) await sleep(30);
  const box = doc.byId['cap-text'];
  const key = (k, extra) => doc.byId['cap-dialog'].fire('keydown', Object.assign({ key: k, target: box, preventDefault() {} }, extra || {}));
  const typed = (v) => { box.value = v; };
  const state = () => ({
    open: !doc.byId.cap.hidden,
    chip: doc.byId['cap-ctx'].hidden ? null : doc.byId['cap-chip-text'].textContent,
    focusIsBox: doc.activeElement === box,
    textLen: box.value.length,
    error: doc.byId['cap-error'].hidden ? null : doc.byId['cap-error'].textContent,
    sent: msgs.map((m) => (m.cmd === 'add' ? { cmd: m.cmd, text: m.text, keep_context: m.keep_context } : { cmd: m.cmd })),
  });
  try {
    await run({ key, typed, state });
    await sleep(30);
    return state();
  } catch (e) {
    return { crash: String(e && e.stack || e) };
  }
}

const CHIP = { repo: 'easy-note-mcp', task: 'HUN-289' };
(async () => {
  const out = {};
  out.start = await scenario(CHIP, async () => {});
  out.enter = await scenario(CHIP, async (s) => { s.typed('fix the tray'); s.key('Enter'); s.key('Enter'); });
  out.shift_enter = await scenario(CHIP, async (s) => { s.typed('line one'); s.key('Enter', { shiftKey: true }); });
  out.composing = await scenario(CHIP, async (s) => { s.typed('日本'); s.key('Enter', { isComposing: true }); });
  out.esc = await scenario(CHIP, async (s) => { s.typed('never mind'); s.key('Escape'); });
  out.empty_enter = await scenario(CHIP, async (s) => { s.typed('  \n '); s.key('Enter'); });
  out.drop_chip = await scenario(CHIP, async (s) => { s.key('Backspace'); s.typed('no context'); s.key('Enter'); });
  out.fast_enter = await scenario(CHIP, async (s) => { s.typed('quick one'); s.key('Enter'); }, true);
  out.no_chip = await scenario(null, async (s) => { s.typed('plain'); s.key('Enter'); });
  // `add` answered with an error: the dialog stays open, the message shows, the text is kept.
  out.add_error = await scenario(CHIP, async (s) => { s.typed('x'.repeat(20001)); s.key('Enter'); }, false, 'the text is too long');
  console.log(JSON.stringify(out));
})();
