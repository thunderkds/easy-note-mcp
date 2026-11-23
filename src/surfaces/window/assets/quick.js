/* The quick window's page (HUN-289): `easy-note quick` in a `window` build shows only the capture
   dialog of capture.js, open from the start. Enter hands the text to Rust (`add`), which saves it the
   way `quick` always did and closes the window; Esc or an empty Enter asks Rust to close it (`close`),
   nothing saved. It sends no other request. */
(function () {
  'use strict';
  function close() { window.easyNoteIpc.close(); }
  var capture = window.easyNoteCapture.attach({ onCancel: close, onEmpty: close });
  capture.open();
})();
