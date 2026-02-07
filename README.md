# RustStream

RustStream is a local streaming web app built in Rust. It uses TMDB for metadata and builds Vidking embed URLs for playback.

## Features

- Browse trending movies and popular TV shows
- Search with filters (genre, year, rating, sort)
- Detailed movie and TV pages with cast and similar titles
- In-browser player using Vidking embed URLs
- Watch history and progress tracking (stored locally)

## Requirements

- Rust toolchain (edition 2021)
- TMDB API v4 read token (Bearer)

## Quick Start

```bash
cargo run --release
```

On first run, a TUI onboarding screen prompts for your TMDB key and writes a `.env` file.

Server starts at `http://127.0.0.1:3000`.

## Downloads

Desktop downloads are published on GitHub Releases for macOS, Windows, and Linux.
After download, run the app and you will be prompted once for your TMDB v4 Read Access Token.

To create a release, tag a version and push the tag:

```bash
git tag v1.0.0
git push origin v1.0.0
```

## Configuration

Environment variables:

- `TMDB_API_KEY` (required)
  - Use your **TMDB v4 Read Access Token** (the long JWT). The short v3 API key will not work.
  - You can set it as either `Bearer <token>` or just the token; the app will add the `Bearer` prefix if missing.
- `DATABASE_URL` (optional, default: `sqlite://./streaming.db`)
- `PORT` (optional, default: `3000`)

## Routes

Pages:

- `GET /` - Home (trending movies + popular TV)
- `GET /search?q=...` - Search page with filters
- `GET /movie/:id` - Movie details
- `GET /tv/:id` - TV details
- `GET /player/:media_type/:id` - Player (TV requires `season` and `episode` query params)
- `GET /history` - Watch history

API:

- `GET /api/movies/popular`
- `GET /api/tv/popular`
- `GET /api/trending/:media_type/:time_window`
- `GET /api/search?q=...`
- `GET /api/movie/:id`
- `GET /api/tv/:id`
- `GET /api/movie/:id/streams`
- `GET /api/tv/:id/streams?season=..&episode=..`
- `POST /api/progress` - Save watch progress (requires login)

## Project Layout

```
streaming/
├── app/
│   ├── src/
│   │   ├── main.rs          # Axum routes + server
│   │   ├── api.rs           # JSON API endpoints
│   │   ├── auth.rs          # Login, sessions, watch history
│   │   ├── config.rs        # Env/config loading
│   │   ├── db.rs            # SQLite schema bootstrap
│   │   ├── models.rs        # Data types
│   │   ├── onboarding.rs    # First-run TUI setup
│   │   ├── templates.rs     # HTML rendering (inline templates)
│   │   ├── tmdb.rs          # TMDB client
│   │   └── vidking.rs       # Vidking embed URLs
│   ├── static/
│   │   └── style.css
│   └── templates/           # Legacy HTML templates (not used)
├── Cargo.toml               # Workspace
└── .env.example
```

## Development

```bash
RUST_LOG=debug cargo run
```

## Notes

- The SQLite database is created automatically on first run.
- Vidking does not require an API key; the app only builds embed URLs.

## License

MIT
