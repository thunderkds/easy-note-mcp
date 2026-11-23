/* The capture dialog of docs/design/Main.dc.html (HUN-280), shared by the dashboard (app.js, `+ New note`)
   and the quick window (quick.js, `easy-note quick`, HUN-289). One dialog per page: the markup ids
   cap, cap-dialog, cap-ctx, cap-chip-text, cap-drop and cap-text. It talks `context` and `add` only.

   window.easyNoteCapture.attach(hooks) wires the dialog once and returns { open, close }. Each hook
   is optional: onOpen(), onClose(), onCancel() (Esc), onEmpty() (Enter with no text, nothing sent),
   onSaved(detail), onError(message). */
(function () {
  'use strict';
  var hooks = {};
  var keepContext = false;
  var capOpenNo = 0;
  var saving = false;

  function $(id) { return document.getElementById(id); }
  function str(v) { return v === undefined || v === null ? '' : String(v); }
  function call(name, arg) { if (typeof hooks[name] === 'function') hooks[name](arg); }

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
    call('onOpen');
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
    call('onClose');
  }

  function capKey(e) {
    var box = $('cap-text');
    if (e.isComposing || e.keyCode === 229) return;
    if (e.key === 'Escape') {
      e.preventDefault();
      closeCapture();
      call('onCancel');
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

  // Empty or whitespace-only text sends nothing. On an error the dialog and its text stay.
  function saveCapture() {
    var text = $('cap-text').value;
    if (!text.trim()) {
      call('onEmpty');
      return;
    }
    if (saving) return;
    saving = true;
    window.easyNoteIpc.add(text, keepContext).then(function (d) {
      saving = false;
      closeCapture();
      call('onSaved', d);
    }, function (err) {
      saving = false;
      call('onError', err.message);
    });
  }

  function attach(given) {
    hooks = given || {};
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
    return { open: openCapture, close: closeCapture };
  }

  window.easyNoteCapture = { attach: attach };
})();
