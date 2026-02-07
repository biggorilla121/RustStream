use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tauri::{Manager, State, WindowUrl};

const DEFAULT_PORT: u16 = 3000;

struct BackendState {
    child: Arc<Mutex<Option<Child>>>,
}

fn main() {
    tauri::Builder::default()
        .manage(BackendState {
            child: Arc::new(Mutex::new(None)),
        })
        .invoke_handler(tauri::generate_handler![save_tmdb_key])
        .setup(|app| {
            let app_handle = app.handle();
            let state = app_handle.state::<BackendState>();
            let child_slot = state.child.clone();

            ensure_default_env(&app_handle).ok();

            if tmdb_key_present(&app_handle) {
                start_backend_and_open_main(app_handle, child_slot);
            } else {
                let _ = tauri::WindowBuilder::new(
                    &app_handle,
                    "setup",
                    WindowUrl::App("setup.html".into()),
                )
                .title("RustStream Setup")
                .inner_size(520.0, 420.0)
                .resizable(false)
                .build();
            }

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

fn start_backend_and_open_main(
    app_handle: tauri::AppHandle,
    child_slot: Arc<Mutex<Option<Child>>>,
) {
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

        if let Some(window) = app_handle.get_window("setup") {
            let _ = window.close();
        }
    });
}

#[tauri::command]
fn save_tmdb_key(
    app: tauri::AppHandle,
    state: State<BackendState>,
    key: String,
) -> Result<(), String> {
    let key = key.trim().to_string();
    if key.is_empty() {
        return Err("TMDB key is required".to_string());
    }

    write_tmdb_key(&app, &key).map_err(|e| e.to_string())?;
    start_backend_and_open_main(app, state.child.clone());
    Ok(())
}

fn tmdb_key_present(app: &tauri::AppHandle) -> bool {
    if let Ok(value) = std::env::var("TMDB_API_KEY") {
        if !value.trim().is_empty() {
            return true;
        }
    }

    read_tmdb_key(app).map(|k| !k.is_empty()).unwrap_or(false)
}

fn read_tmdb_key(app: &tauri::AppHandle) -> Option<String> {
    let env_path = default_env_path(app).ok()?;
    let contents = std::fs::read_to_string(env_path).ok()?;
    for line in contents.lines() {
        if let Some(value) = line.strip_prefix("TMDB_API_KEY=") {
            return Some(value.trim().to_string());
        }
    }
    None
}

fn write_tmdb_key(app: &tauri::AppHandle, key: &str) -> anyhow::Result<()> {
    let env_path = default_env_path(app)?;
    if let Some(parent) = env_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut lines = Vec::new();
    let mut replaced = false;

    if let Ok(contents) = std::fs::read_to_string(&env_path) {
        for line in contents.lines() {
            if line.starts_with("TMDB_API_KEY=") {
                lines.push(format!("TMDB_API_KEY={}", key));
                replaced = true;
            } else if !line.trim().is_empty() {
                lines.push(line.to_string());
            }
        }
    }

    if !replaced {
        lines.push(format!("TMDB_API_KEY={}", key));
    }

    std::fs::write(env_path, lines.join("\n") + "\n")?;
    Ok(())
}

fn read_port() -> u16 {
    std::env::var("PORT")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(DEFAULT_PORT)
}

fn spawn_backend(app: &tauri::AppHandle, port: u16) -> anyhow::Result<Child> {
    ensure_default_env(app)?;
    let database_url = build_database_url(app)?;

    if let Some(path) = resolve_packaged_backend(app) {
        let env_path = default_env_path(app)?;
        return spawn_command(path, port, Some(database_url), Some(env_path));
    }

    if let Ok(path) = std::env::var("RUSTSTREAM_BACKEND") {
        let env_path = default_env_path(app)?;
        return spawn_command(PathBuf::from(path), port, Some(database_url), Some(env_path));
    }

    if let Some(path) = resolve_local_backend_near_exe() {
        let env_path = default_env_path(app)?;
        return spawn_command(path, port, Some(database_url), Some(env_path));
    }

    if let Some(path) = resolve_workspace_backend() {
        let env_path = default_env_path(app)?;
        return spawn_command(path, port, Some(database_url), Some(env_path));
    }

    anyhow::bail!("Unable to locate backend binary");
}

fn spawn_command(
    path: PathBuf,
    port: u16,
    database_url: Option<String>,
    env_path: Option<PathBuf>,
) -> anyhow::Result<Child> {
    let mut cmd = Command::new(path);
    cmd.env("PORT", port.to_string());
    if let Some(url) = database_url {
        cmd.env("DATABASE_URL", url);
    }
    if let Some(path) = env_path {
        cmd.env("DOTENVY_FILENAME", path);
    }
    cmd.spawn().map_err(|e| e.into())
}

fn resolve_packaged_backend(app: &tauri::AppHandle) -> Option<PathBuf> {
    let resource_dir = tauri::api::path::resource_dir(app.package_info(), &app.env())?;
    let candidate = resource_dir.join("bin").join(backend_binary_name());
    if candidate.exists() {
        Some(candidate)
    } else {
        None
    }
}

fn resolve_local_backend_near_exe() -> Option<PathBuf> {
    let exe_path = std::env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?;
    let candidate = exe_dir.join("bin").join(backend_binary_name());
    if candidate.exists() {
        return Some(candidate);
    }

    let sibling = exe_dir.join(backend_binary_name());
    if sibling.exists() {
        return Some(sibling);
    }

    None
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

fn build_database_url(app: &tauri::AppHandle) -> anyhow::Result<String> {
    let data_dir = tauri::api::path::app_data_dir(&app.config())
        .ok_or_else(|| anyhow::anyhow!("Unable to resolve app data directory"))?;
    std::fs::create_dir_all(&data_dir)?;
    let db_path = data_dir.join("streaming.db");
    Ok(db_path.to_string_lossy().to_string())
}

fn default_env_path(app: &tauri::AppHandle) -> anyhow::Result<PathBuf> {
    let data_dir = tauri::api::path::app_data_dir(&app.config())
        .ok_or_else(|| anyhow::anyhow!("Unable to resolve app data directory"))?;
    Ok(data_dir.join(".env"))
}

fn ensure_default_env(app: &tauri::AppHandle) -> anyhow::Result<()> {
    let env_path = default_env_path(app)?;
    if let Some(parent) = env_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if env_path.exists() {
        return Ok(());
    }

    std::fs::write(env_path, "TMDB_API_KEY=\n")?;
    Ok(())
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
