use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tauri::{Manager, WindowUrl};

const DEFAULT_PORT: u16 = 3000;

struct BackendState {
    child: Arc<Mutex<Option<Child>>>,
}

fn main() {
    tauri::Builder::default()
        .manage(BackendState {
            child: Arc::new(Mutex::new(None)),
        })
        .setup(|app| {
            let app_handle = app.handle();
            let state = app_handle.state::<BackendState>();
            let child_slot = state.child.clone();

            std::thread::spawn(move || {
                let port = read_port();
                if !is_port_open(port) {
                    match spawn_backend(&app_handle, port) {
                        Ok(child) => {
                            *child_slot.lock().expect("backend lock") = Some(child);
                        }
                        Err(err) => {
                            eprintln!("Failed to start backend: {err}");
                            return;
                        }
                    }
                }

                if !wait_for_port(port, Duration::from_secs(20)) {
                    eprintln!("Backend did not become ready on port {port}");
                    return;
                }

                let url = format!("http://127.0.0.1:{port}");
                let _ = tauri::WindowBuilder::new(
                    &app_handle,
                    "main",
                    WindowUrl::External(url.parse().expect("valid url")),
                )
                .title("RustStream")
                .build();
            });

            Ok(())
        })
        .on_window_event(|event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event.event() {
                if let Some(state) = event.window().app_handle().try_state::<BackendState>() {
                    if let Some(mut child) = state.child.lock().ok().and_then(|mut c| c.take()) {
                        let _ = child.kill();
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn read_port() -> u16 {
    std::env::var("PORT")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(DEFAULT_PORT)
}

fn spawn_backend(app: &tauri::AppHandle, port: u16) -> anyhow::Result<Child> {
    if let Ok(path) = std::env::var("RUSTSTREAM_BACKEND") {
        return spawn_command(PathBuf::from(path), port);
    }

    if let Some(path) = resolve_packaged_backend(app) {
        return spawn_command(path, port);
    }

    if let Some(path) = resolve_workspace_backend() {
        return spawn_command(path, port);
    }

    anyhow::bail!("Unable to locate backend binary");
}

fn spawn_command(path: PathBuf, port: u16) -> anyhow::Result<Child> {
    let mut cmd = Command::new(path);
    cmd.env("PORT", port.to_string());
    cmd.spawn().map_err(|e| e.into())
}

fn resolve_packaged_backend(app: &tauri::AppHandle) -> Option<PathBuf> {
    let resource_dir = tauri::api::path::resource_dir(app.package_info(), &app.env())?;
    let candidate = resource_dir.join(backend_binary_name());
    if candidate.exists() {
        Some(candidate)
    } else {
        None
    }
}

fn resolve_workspace_backend() -> Option<PathBuf> {
    let mut root = std::env::current_dir().ok()?;
    if root.ends_with("src-tauri") {
        root.pop();
        root.pop();
    }

    let debug_path = root
        .join("target")
        .join("debug")
        .join(backend_binary_name());
    if debug_path.exists() {
        return Some(debug_path);
    }

    let release_path = root
        .join("target")
        .join("release")
        .join(backend_binary_name());
    if release_path.exists() {
        return Some(release_path);
    }

    None
}

fn backend_binary_name() -> &'static str {
    if cfg!(windows) {
        "ruststream.exe"
    } else {
        "ruststream"
    }
}

fn is_port_open(port: u16) -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok()
}

fn wait_for_port(port: u16, timeout: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if is_port_open(port) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    false
}
