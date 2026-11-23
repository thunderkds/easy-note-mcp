/* The quick window's page (HUN-289): `easy-note quick` in a `window` build shows only the capture
   dialog of capture.js, open from the start. Enter hands the text to Rust (`add`), which saves it the
   way `quick` always did and closes the window; Esc or an empty Enter asks Rust to close it (`close`),
   nothing saved. It sends no other request. An error reply to `add` (as "the text is too long") shows
   in the error line; the dialog and the text stay. */
(function () {
  'use strict';
  function close() { window.easyNoteIpc.close(); }
  function showError(message) {
    var line = document.getElementById('cap-error');
    line.textContent = message;
    line.hidden = false;
  }
  var capture = window.easyNoteCapture.attach({ onCancel: close, onEmpty: close, onError: showError });
  capture.open();
})();
