/* The Settings dialog of the dashboard (HUN-300, FR-052): `[ai]` of config.toml, read and saved with
   `settings_get` and `settings_save` only. The core judges every value; this page sends what is typed.

   window.easyNoteSettings.attach(hooks) wires the gear and the two dialogs once. Hooks: onOpen(),
   onClose(), toast(message, ms).

   NFR-014: the "what is sent" notice is shown before scheduled runs turn on, unless the same text
   (account folder and model) was confirmed before. The switch changes only on Confirm. */
(function () {
  'use strict';
  var NUMBERS = [['set-interval', 'interval_min'], ['set-max', 'max_per_run'], ['set-batch', 'batch_size']];
  var JOBS = ['organize', 'drafts', 'digest'];
  var ERROR_MS = 6000;
  var hooks = {};
  var loaded = null; // the last `settings_get` / `settings_save` answer
  var draft = { enrich: false, schedule: false };
  var saving = false;
  var openNo = 0;
  var closeAfterConfirm = false;

  function $(id) { return document.getElementById(id); }

  // One of 'loading', 'form' or 'failed'.
  function show(part) {
    $('set-loading').hidden = part !== 'loading';
    $('set-form').hidden = part !== 'form';
    $('set-save').hidden = part !== 'form';
    $('set-retry').hidden = part !== 'failed';
  }

  function showError(message) {
    $('set-error').textContent = message || '';
    $('set-error').hidden = !message;
  }

  function renderSwitches() {
    $('set-enrich').setAttribute('aria-checked', String(draft.enrich));
    $('set-schedule').setAttribute('aria-checked', String(draft.enrich && draft.schedule));
    $('set-schedule').disabled = !draft.enrich;
    $('set-schedule-reason').hidden = draft.enrich;
  }

  function savedOn() {
    return loaded.settings.enrich === 'claude' && loaded.settings.schedule === 'tray';
  }

  function fill(answer) {
    loaded = answer;
    var s = answer.settings;
    draft.enrich = s.enrich === 'claude';
    draft.schedule = savedOn();
    NUMBERS.forEach(function (n) { $(n[0]).value = String(s[n[1]]); });
    JOBS.forEach(function (job) { $('set-job-' + job).checked = s.jobs.indexOf(job) >= 0; });
    $('set-model').value = s.model;
    $('set-after-save').checked = !!s.after_save;
    $('set-path').textContent = answer.path;
    $('set-command').textContent = answer.command;
    $('set-account').textContent = answer.account;
    renderSwitches();
  }

  function model() { return $('set-model').value.trim(); }

  // A number goes as a number; anything else goes as typed, and the core refuses it by its key.
  function number(id) {
    var typed = $(id).value.trim();
    return /^\d+$/.test(typed) ? Number(typed) : typed;
  }

  function values() {
    var out = {
      enrich: draft.enrich ? 'claude' : 'off',
      schedule: draft.enrich && draft.schedule ? 'tray' : 'off'
    };
    NUMBERS.forEach(function (n) { out[n[1]] = number(n[0]); });
    out.jobs = JOBS.filter(function (job) { return $('set-job-' + job).checked; });
    out.model = model();
    out.after_save = !!$('set-after-save').checked;
    return out;
  }

  // Turning scheduled runs on shows the notice, unless this account and model were confirmed.
  function needsNotice() {
    return !savedOn() && model() !== loaded.notice.confirmed_model;
  }

  // `closeAfter`: the notice was raised by Save, so Confirm finishes that save.
  function ask(closeAfter) {
    closeAfterConfirm = closeAfter;
    var account = loaded.account;
    var name = model();
    $('ask-text').textContent = loaded.notice.template
      .replace('{dir}', function () { return account; })
      .replace('{model}', function () { return name; });
    $('set').inert = true;
    $('ask').hidden = false;
    $('ask-cancel').focus();
  }

  function closeAsk() {
    $('ask').hidden = true;
    $('set').inert = false;
  }

  function cancelAsk() {
    if ($('ask').hidden) return;
    closeAsk();
    draft.schedule = false;
    renderSwitches();
    $('set-schedule').focus();
  }

  function confirmAsk() {
    if ($('ask').hidden) return;
    closeAsk();
    draft.schedule = true;
    renderSwitches();
    $('set-schedule').focus();
    save(true, closeAfterConfirm);
  }

  function load(note) {
    openNo += 1;
    var no = openNo;
    show('loading');
    showError('');
    window.easyNoteIpc.settingsGet().then(function (answer) {
      if (no !== openNo) return;
      fill(answer);
      show('form');
      showError(note);
      $('set-enrich').focus();
    }, function (err) {
      if (no !== openNo) return;
      show('failed');
      showError('Could not read the settings: ' + err.message + '. Fix the file, then press Try again.');
      $('set-retry').focus();
    });
  }

  function save(confirmed, closeAfter) {
    if (saving) return;
    saving = true;
    var no = openNo;
    $('set-save').disabled = true;
    showError('');
    function done() {
      saving = false;
      $('set-save').disabled = false;
    }
    window.easyNoteIpc.settingsSave(loaded.base, values(), confirmed).then(function (answer) {
      done();
      hooks.toast('Saved');
      if (no !== openNo) return;
      fill(answer);
      if (closeAfter) close();
    }, function (err) {
      done();
      var message = 'Not saved: ' + err.message;
      hooks.toast(message, ERROR_MS);
      if (no !== openNo) return;
      // As Edit does: nothing is overwritten; the screen reads the file again.
      if (err.message.indexOf('changed outside') >= 0) {
        load(message + '. These are the values in the file now. Change them again, then press Save.');
        return;
      }
      // Nothing was saved, so a switch turned on by Confirm goes back off.
      if (confirmed) {
        draft.schedule = false;
        renderSwitches();
      }
      showError(message);
    });
  }

  function pressSave() {
    if ($('set-form').hidden || saving) return;
    if (draft.enrich && draft.schedule && needsNotice()) ask(true);
    else save(false, true);
  }

  function open() {
    if (!$('set').hidden) return;
    closeAsk();
    $('set').hidden = false;
    hooks.onOpen();
    $('set-close').focus();
    load('');
  }

  function close() {
    openNo += 1;
    closeAsk();
    $('set').hidden = true;
    hooks.onClose();
  }

  function onEscape(action) {
    return function (e) {
      if (e.key !== 'Escape' || e.isComposing) return;
      e.preventDefault();
      action();
    };
  }

  function attach(given) {
    hooks = given;
    $('settings-open').addEventListener('click', open);
    $('set-close').addEventListener('click', close);
    $('set-retry').addEventListener('click', function () { load(''); });
    $('set-save').addEventListener('click', pressSave);
    $('set-enrich').addEventListener('click', function () {
      draft.enrich = !draft.enrich;
      renderSwitches();
    });
    $('set-schedule').addEventListener('click', function () {
      if (!draft.enrich || saving) return;
      if (!draft.schedule && needsNotice()) {
        ask(false);
        return;
      }
      draft.schedule = !draft.schedule;
      renderSwitches();
    });
    NUMBERS.map(function (n) { return n[0]; }).concat('set-model').forEach(function (id) {
      $(id).addEventListener('keydown', function (e) {
        if (e.key !== 'Enter' || e.isComposing || e.keyCode === 229) return;
        e.preventDefault();
        pressSave();
      });
    });
    $('ask-confirm').addEventListener('click', confirmAsk);
    $('ask-cancel').addEventListener('click', cancelAsk);
    $('set-dialog').addEventListener('keydown', onEscape(close));
    $('ask-dialog').addEventListener('keydown', onEscape(cancelAsk));
    // A click on a backdrop would move focus to the page, where Escape reaches no handler.
    ['set', 'ask'].forEach(function (id) {
      $(id).addEventListener('mousedown', function (e) {
        if (e.target === $(id)) e.preventDefault();
      });
    });
  }

  window.easyNoteSettings = { attach: attach };
})();
