# RustStream

A fully Rust-based streaming website using TMDB API for movie metadata and Vidking API for streaming links.

## Features

- **Browse Movies & TV Shows**: View trending and popular content
- **Search**: Find movies and TV shows by title
- **Detailed Views**: See cast, similar content, ratings, and more
- **Video Player**: Watch content directly in the browser
- **Responsive Design**: Works on desktop and mobile

## Tech Stack

- **Backend**: Axum (Rust web framework)
- **Database**: SQLite (local file-based)
- **APIs**: TMDB (metadata), Vidking (streaming)
- **Frontend**: Server-side rendered HTML with CSS

## Installation

### Prerequisites

- Rust (already installed on your system)
- TMDB API key
- Vidking API key

### Step 1: Clone/Navigate to the project

```bash
cd streaming
```

### Step 2: Set up environment variables

Copy the example environment file:

```bash
cp .env.example .env
```

Edit `.env` and add your API key:

```env
TMDB_API_KEY=your_tmdb_api_key_here
```

To get your API key:
- **TMDB**: Sign up at https://www.themoviedb.org/settings/api
Vidking does not require an API key.

### Step 3: Build the project

```bash
cargo build --release
```

### Step 4: Run the server

```bash
cargo run --release
```

Or run the binary directly:

```bash
./target/release/streaming-app
```

The server will start at: http://127.0.0.1:3000

## Project Structure

```
streaming/
├── app/
│   ├── src/
│   │   ├── main.rs          # Main entry point
│   │   ├── api.rs           # REST API routes
│   │   ├── config.rs        # Configuration management
│   │   ├── db.rs            # Database initialization
│   │   ├── error.rs         # Error handling
│   │   ├── models.rs        # Data structures
│   │   ├── tmdb.rs          # TMDB API client
│   │   ├── vidking.rs       # Vidking API client
│   │   └── templates.rs     # HTML template rendering
│   ├── static/
│   │   └── style.css        # Styling
│   └── templates/           # (Not used - inline HTML)
├── Cargo.toml              # Workspace configuration
└── .env.example            # Environment variables template
```

## API Endpoints

- `GET /` - Home page with trending movies and popular TV shows
- `GET /search?q={query}` - Search for movies and TV shows
- `GET /movie/{id}` - Movie details page
- `GET /tv/{id}` - TV show details page
- `GET /player/{media_type}/{id}` - Video player
- `GET /api/movies/popular` - Get popular movies (JSON)
- `GET /api/tv/popular` - Get popular TV shows (JSON)
- `GET /api/search?q={query}` - Search API (JSON)
- `GET /api/movie/{id}` - Get movie details (JSON)
- `GET /api/tv/{id}` - Get TV show details (JSON)
- `GET /api/movie/{id}/streams` - Get movie streaming links
- `GET /api/tv/{id}/streams?season={n}&episode={n}` - Get TV episode streaming links

## Configuration

Environment variables:
- `TMDB_API_KEY` - Your TMDB API key (required)
- `VIDKING_API_KEY` - Your Vidking API key (required)
- `DATABASE_URL` - SQLite database path (default: `sqlite://./streaming.db`)
- `PORT` - Server port (default: `3000`)

## Notes

- The database is automatically created on first run
- Placeholder images are used when posters are unavailable
- The video player uses browser's native HTML5 video player
- Stream quality and availability depend on Vidking API

## Troubleshooting

If you get compilation errors, try:
```bash
cargo clean
cargo build --release
```

If the server won't start, check that:
1. Your API keys are set correctly in `.env`
2. Port 3000 is not already in use
3. You have internet connectivity for API calls

## Development

To run in development mode with logging:

```bash
RUST_LOG=debug cargo run
```

## License

MIT
