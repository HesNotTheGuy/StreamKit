use axum::Router;
use std::path::PathBuf;
use std::sync::mpsc;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Theme, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_window_state::{AppHandleExt, StateFlags, WindowExt};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

/// Start the Axum file server. Binds immediately so the port is live before
/// the Tauri window is created, then signals the bound port (or bind error)
/// over the channel. Port 3001 is required — OBS overlay URLs and the Twitch
/// OAuth redirect URI are hard-coded to it, so we do NOT fall back to a random
/// port. If 3001 is busy, we report the error and let main exit loudly.
async fn start_file_server(
    base_dir: PathBuf,
    port_tx: mpsc::Sender<Result<u16, String>>,
) {
    let cors = CorsLayer::new().allow_origin(Any);

    let router = Router::new()
        .nest_service("/src", ServeDir::new(base_dir.join("src")))
        .nest_service("/assets", ServeDir::new(base_dir.join("assets")))
        .layer(cors);

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
            let (tx, rx) = mpsc::channel::<Result<u16, String>>();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(start_file_server(base_dir, tx));
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
