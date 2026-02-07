# RustStream Desktop (Tauri)

This is a lightweight native shell that hosts the existing RustStream web UI
inside the system WebView (no Electron). It keeps the Vidking iframe intact.

## How it works

- The desktop app launches the Rust backend (`ruststream`) as a child process.
- It waits for the local server to be ready, then opens a native window to it.

## Dev Run

1. Build the backend binary:

```bash
cargo build -p streaming-app
```

2. Run the desktop app:

```bash
cd desktop/src-tauri
cargo run
```

## Notes

- You can point the desktop app at a custom backend binary:

```bash
RUSTSTREAM_BACKEND=/path/to/ruststream cargo run
```

- The window loads `http://127.0.0.1:3000` by default (controlled by `PORT`).
