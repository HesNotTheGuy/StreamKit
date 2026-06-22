use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Theme, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_window_state::{AppHandleExt, StateFlags, WindowExt};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

/// Shared state for the local control API (Stream Deck / Touch Portal / etc.).
/// Window actions run directly in Rust; all other actions are queued for the
/// frontend, which drains them via `GET /api/actions`.
struct ControlState {
    queue: Mutex<VecDeque<String>>,
    token: String,
    app: tauri::AppHandle,
}

/// Generate a per-launch token. Not cryptographically strong, but enough to keep
/// a stray local web page from blindly triggering actions on the loopback server.
fn gen_token() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id() as u128;
    let mixed = nanos ^ pid.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ nanos.rotate_left(40);
    format!("{:032x}", mixed)
}

/// Load the persisted control token, or generate + persist one on first run.
/// Persisting keeps controller URLs (which embed the token) valid across restarts.
fn load_or_create_token(app: &tauri::AppHandle) -> String {
    let dir = match app.path().app_config_dir() {
        Ok(d) => d,
        Err(_) => return gen_token(),
    };
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("remote_token.txt");
    if let Ok(existing) = std::fs::read_to_string(&path) {
        let t = existing.trim().to_string();
        if !t.is_empty() {
            return t;
        }
    }
    let t = gen_token();
    let _ = std::fs::write(&path, &t);
    t
}

/// `GET /api/token` — the app's own Remote page reads this to build controller URLs.
/// No CORS layer is applied to /api, so cross-origin pages cannot read the response.
async fn get_token(State(s): State<Arc<ControlState>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "token": s.token }))
}

/// `GET|POST /api/action/{name}?token=…` — trigger an action from an external controller.
async fn do_action(
    State(s): State<Arc<ControlState>>,
    Path(name): Path<String>,
    Query(q): Query<HashMap<String, String>>,
) -> (StatusCode, Json<serde_json::Value>) {
    if q.get("token").map(String::as_str) != Some(s.token.as_str()) {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "ok": false, "error": "invalid or missing token" })),
        );
    }

    match name.as_str() {
        "show_window" | "hide_window" | "toggle_window" => {
            if let Some(w) = s.app.get_webview_window("main") {
                match name.as_str() {
                    "show_window" => {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                    "hide_window" => {
                        let _ = w.hide();
                    }
                    _ => {
                        if w.is_visible().unwrap_or(false) {
                            let _ = w.hide();
                        } else {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                }
            }
        }
        _ => {
            let mut q = s.queue.lock().unwrap();
            if q.len() >= 50 {
                q.pop_front();
            }
            q.push_back(name.clone());
        }
    }

    (StatusCode::OK, Json(serde_json::json!({ "ok": true, "action": name })))
}

/// `GET /api/actions?token=…` — the frontend polls this and drains the queue.
async fn drain_actions(
    State(s): State<Arc<ControlState>>,
    Query(q): Query<HashMap<String, String>>,
) -> (StatusCode, Json<serde_json::Value>) {
    if q.get("token").map(String::as_str) != Some(s.token.as_str()) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "actions": [] })));
    }
    let actions: Vec<String> = {
        let mut queue = s.queue.lock().unwrap();
        queue.drain(..).collect()
    };
    (StatusCode::OK, Json(serde_json::json!({ "actions": actions })))
}

/// Start the Axum file server. Binds immediately so the port is live before
/// the Tauri window is created, then signals the bound port (or bind error)
/// over the channel. Port 3001 is required — OBS overlay URLs and the Twitch
/// OAuth redirect URI are hard-coded to it, so we do NOT fall back to a random
/// port. If 3001 is busy, we report the error and let main exit loudly.
async fn start_file_server(
    base_dir: PathBuf,
    app: tauri::AppHandle,
    token: String,
    port_tx: mpsc::Sender<Result<u16, String>>,
) {
    let cors = CorsLayer::new().allow_origin(Any);

    let state = Arc::new(ControlState {
        queue: Mutex::new(VecDeque::new()),
        token,
        app,
    });

    // Static file serving (CORS-open for OBS). /api routes deliberately omit CORS.
    let files = Router::new()
        .nest_service("/src", ServeDir::new(base_dir.join("src")))
        .nest_service("/assets", ServeDir::new(base_dir.join("assets")))
        .layer(cors);

    let router = Router::new()
        .route("/api/token", get(get_token))
        .route("/api/action/:name", get(do_action).post(do_action))
        .route("/api/actions", get(drain_actions))
        .with_state(state)
        .merge(files);

    let listener = match tokio::net::TcpListener::bind("127.0.0.1:3001").await {
        Ok(l) => l,
        Err(e) => {
            let _ = port_tx.send(Err(format!(
                "StreamKit requires port 3001, but it is already in use ({}). \
                 Close the other application using port 3001 and try again.",
                e
            )));
            return;
        }
    };

    let port = listener.local_addr().unwrap().port();
    let _ = port_tx.send(Ok(port)); // unblocks main thread; server is now accepting

    if let Err(e) = axum::serve(listener, router).await {
        eprintln!("File server crashed: {}", e);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            // ── Resolve base directory ─────────────────────────────────────────
            let base_dir = if cfg!(dev) {
                // dev: CARGO_MANIFEST_DIR = src-tauri/, so go up one level
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .unwrap()
                    .to_path_buf()
            } else {
                // prod: installer places src/ and assets/ next to the .exe
                std::env::current_exe()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .to_path_buf()
            };

            // ── Start HTTP server and wait until it's listening ────────────────
            let app_handle = app.handle().clone();
            let token = load_or_create_token(&app_handle);
            let (tx, rx) = mpsc::channel::<Result<u16, String>>();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(start_file_server(base_dir, app_handle, token, tx));
            });
            let port = match rx.recv() {
                Ok(Ok(p)) => p,
                Ok(Err(msg)) => {
                    eprintln!("StreamKit startup failed: {}", msg);
                    app.handle().exit(1);
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("StreamKit startup failed: HTTP server thread died: {}", e);
                    app.handle().exit(1);
                    return Ok(());
                }
            };

            // ── Main window ────────────────────────────────────────────────────
            let url = format!("http://localhost:{}/src/main.html", port);
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url.parse().unwrap()))
                .title("StreamKit")
                .inner_size(1280.0, 780.0)
                .min_inner_size(960.0, 600.0)
                .resizable(true)
                .decorations(true)
                .theme(Some(Theme::Dark)) // dark title bar on Windows 11
                .build()?;

            // Restore saved size + position (if any)
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.restore_state(StateFlags::all());
            }

            // ── System tray ────────────────────────────────────────────────────
            let open_item =
                MenuItem::with_id(app, "open", "Open StreamKit", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let quit_item =
                MenuItem::with_id(app, "quit", "Quit StreamKit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_item, &sep, &quit_item])?;

            let icon = Image::from_bytes(include_bytes!("../icons/icon.png"))?;

            TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .tooltip("StreamKit")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    "quit" => {
                        // Save window state before exiting
                        let _ = app.save_window_state(StateFlags::all());
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // Left-click the tray icon → show/focus window
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        // X button hides to tray instead of closing
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.app_handle().save_window_state(StateFlags::all());
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running StreamKit");
}
