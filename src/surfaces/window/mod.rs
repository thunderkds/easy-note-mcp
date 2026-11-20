//! easy-note's own dashboard window (FR-042, DDR 0009): a tao window with a wry (WebKitGTK)
//! webview, behind the `window` cargo feature. The page is embedded in the binary and served
//! only through the `easy-note://` scheme; it reaches the notes only through IPC answered by the
//! pure `api::handle`. The CSP, the asset table, the URL check and the reply script are plain
//! functions here so the default build tests them; only the shell needs the feature.

pub mod api;

use serde_json::Value;

use crate::core::store::Store;

/// The Content-Security-Policy of every page response: nothing but the app's own scheme.
pub const CSP: &str = "default-src easy-note:; style-src easy-note: 'unsafe-inline'";
/// The page's address.
pub const START: &str = "easy-note://localhost/";
/// What the shell runs when the window gets focus: the page asks `board` again.
pub const REFRESH: &str = "window.__easyNote && window.__easyNote.refresh()";

const HTML: &str = "text/html; charset=utf-8";
const JS: &str = "text/javascript; charset=utf-8";
const CSS: &str = "text/css; charset=utf-8";
const WOFF2: &str = "font/woff2";

/// The embedded file for a request path, with its MIME type; `None` for any other path.
pub fn asset(path: &str) -> Option<(&'static str, &'static [u8])> {
    match path {
        "/" | "/index.html" => Some((HTML, include_bytes!("assets/index.html"))),
        "/app.js" => Some((JS, include_bytes!("assets/app.js"))),
        "/app.css" => Some((CSS, include_bytes!("assets/app.css"))),
        "/ipc.js" => Some((JS, include_bytes!("assets/ipc.js"))),
        "/fonts/ibm-plex-sans-latin-400-normal.woff2" => Some((
            WOFF2,
            include_bytes!("assets/fonts/ibm-plex-sans-latin-400-normal.woff2"),
        )),
        "/fonts/ibm-plex-sans-latin-500-normal.woff2" => Some((
            WOFF2,
            include_bytes!("assets/fonts/ibm-plex-sans-latin-500-normal.woff2"),
        )),
        "/fonts/ibm-plex-sans-latin-600-normal.woff2" => Some((
            WOFF2,
            include_bytes!("assets/fonts/ibm-plex-sans-latin-600-normal.woff2"),
        )),
        "/fonts/ibm-plex-mono-latin-400-normal.woff2" => Some((
            WOFF2,
            include_bytes!("assets/fonts/ibm-plex-mono-latin-400-normal.woff2"),
        )),
        "/fonts/ibm-plex-mono-latin-500-normal.woff2" => Some((
            WOFF2,
            include_bytes!("assets/fonts/ibm-plex-mono-latin-500-normal.woff2"),
        )),
        _ => None,
    }
}

/// Navigation is allowed only inside the app's own scheme.
pub fn allowed_url(url: &str) -> bool {
    url.starts_with("easy-note://")
}

/// The script that hands one `api::handle` answer to the page: the JSON goes in as one JS string
/// literal (serde_json escapes it), never spliced in raw.
pub fn reply_script(answer: &str) -> String {
    format!(
        "window.__easyNote.reply({})",
        Value::String(answer.to_string())
    )
}

/// One IPC message from the page: `{"req": <number>, "cmd": ..., ...}`. `req` is taken out, the
/// rest goes to `api::handle`, and the same `req` is put back at the top level of the answer
/// (an error too), so the page matches each reply to its request. A `req` that is not a number
/// is dropped, never echoed.
pub fn ipc(store: &Store, body: &str) -> String {
    ipc_with(store, body, &|_| {})
}

/// `ipc`, with the `api::handle_with` hook for the id of a saved note.
pub fn ipc_with(store: &Store, body: &str, on_added: &dyn Fn(&str)) -> String {
    let Ok(mut request) = serde_json::from_str::<Value>(body) else {
        return api::handle_with(store, body, on_added);
    };
    let req = request
        .as_object_mut()
        .and_then(|o| o.remove("req"))
        .filter(Value::is_number);
    let answer = api::handle_with(store, &request.to_string(), on_added);
    let Some(req) = req else { return answer };
    match serde_json::from_str::<Value>(&answer) {
        Ok(Value::Object(mut map)) => {
            map.insert("req".to_string(), req);
            Value::Object(map).to_string()
        }
        _ => answer,
    }
}

/// Without the feature, `dashboard` never asks for the window (`dashboard::use_window`).
#[cfg(not(feature = "window"))]
pub fn open() -> Result<(), String> {
    Err("this easy-note is built without the window feature".to_string())
}

#[cfg(feature = "window")]
pub use shell::open;

#[cfg(feature = "window")]
mod shell {
    use std::borrow::Cow;

    use tao::dpi::LogicalSize;
    use tao::event::{Event, WindowEvent};
    use tao::event_loop::{ControlFlow, EventLoopBuilder};
    use tao::platform::unix::WindowExtUnix;
    use tao::window::WindowBuilder;
    use wry::http::{header, Response, StatusCode};
    use wry::{NewWindowResponse, WebViewBuilder, WebViewBuilderExtUnix};

    use super::{allowed_url, asset, ipc_with, reply_script, CSP, REFRESH, START};
    use crate::core::config;
    use crate::core::store::Store;
    use crate::surfaces::enrich;

    /// Some GPU / Wayland setups draw a blank WebKitGTK window without it (DDR 0009).
    const DMABUF: &str = "WEBKIT_DISABLE_DMABUF_RENDERER";

    /// The person's own value wins; set ours only when the variable is unset.
    pub(super) fn needs_dmabuf_default(current: Option<&std::ffi::OsStr>) -> bool {
        current.is_none()
    }

    fn respond(path: &str) -> Response<Cow<'static, [u8]>> {
        let (status, mime, body): (StatusCode, &str, &'static [u8]) = match asset(path) {
            Some((mime, body)) => (StatusCode::OK, mime, body),
            None => (StatusCode::NOT_FOUND, "text/plain; charset=utf-8", b""),
        };
        Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, mime)
            .header(header::CONTENT_SECURITY_POLICY, CSP)
            .body(Cow::Borrowed(body))
            .unwrap_or_else(|_| Response::new(Cow::Borrowed(b"")))
    }

    /// Open the window and run its event loop until it is closed (the process then exits).
    pub fn open() -> Result<(), String> {
        // One config read at start: the same `[ai] enrich` + `after_save` rule as `easy-note add`.
        let (dir, after_save) = config::notes_dir_and_after_save().map_err(|e| e.to_string())?;
        let store = Store::system(dir);
        if needs_dmabuf_default(std::env::var_os(DMABUF).as_deref()) {
            // Before any GTK or WebKit code runs; still one thread here.
            std::env::set_var(DMABUF, "1");
        }
        let event_loop = EventLoopBuilder::<String>::with_user_event().build();
        let proxy = event_loop.create_proxy();
        let window = WindowBuilder::new()
            .with_title("easy-note")
            .with_inner_size(LogicalSize::new(1280.0, 820.0))
            .with_min_inner_size(LogicalSize::new(800.0, 560.0))
            .build(&event_loop)
            .map_err(|e| e.to_string())?;
        let vbox = window
            .default_vbox()
            .ok_or_else(|| "no GTK container for the webview".to_string())?;
        let webview = WebViewBuilder::new()
            .with_custom_protocol("easy-note".into(), |_, request| {
                respond(request.uri().path())
            })
            .with_ipc_handler(move |request| {
                // The answer goes back through the event loop, where the webview lives.
                let answer = ipc_with(&store, request.body(), &|id| {
                    if after_save {
                        enrich::after_save(&store.root, id);
                    }
                });
                let _ = proxy.send_event(answer);
            })
            .with_navigation_handler(|url| allowed_url(&url))
            .with_new_window_req_handler(|_, _| NewWindowResponse::Deny)
            .with_devtools(cfg!(debug_assertions))
            .with_url(START)
            .build_gtk(vbox)
            .map_err(|e| e.to_string())?;
        event_loop.run(move |event, _, flow| {
            *flow = ControlFlow::Wait;
            let _ = &window;
            match event {
                Event::UserEvent(answer) => {
                    let _ = webview.evaluate_script(&reply_script(&answer));
                }
                Event::WindowEvent {
                    event: WindowEvent::Focused(true),
                    ..
                } => {
                    let _ = webview.evaluate_script(REFRESH);
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *flow = ControlFlow::Exit,
                _ => {}
            }
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::ffi::OsStr;

        #[test]
        fn dmabuf_is_set_only_when_unset() {
            assert!(needs_dmabuf_default(None));
            assert!(!needs_dmabuf_default(Some(OsStr::new("0"))));
            assert!(!needs_dmabuf_default(Some(OsStr::new("1"))));
            assert!(!needs_dmabuf_default(Some(OsStr::new(""))));
        }

        #[test]
        fn every_response_carries_the_csp() {
            for path in ["/", "/app.js", "/app.css", "/ipc.js", "/missing"] {
                let r = respond(path);
                assert_eq!(r.headers()[header::CONTENT_SECURITY_POLICY], CSP, "{path}");
            }
            assert_eq!(respond("/").status(), StatusCode::OK);
            assert_eq!(respond("/missing").status(), StatusCode::NOT_FOUND);
        }
    }
}
