use crate::tmdb::{Genre, MovieDetail, SearchResult, TvShowDetail};
use crate::vidking::StreamSource;

pub fn render_login(username: Option<&str>, error: Option<&str>) -> String {
    let mut html = String::new();
    html.push_str(&base_start("Login - RustStream", username));

    html.push_str(
        r#"
    <div class="login-page">
        <h1>Admin Login</h1>
        <div class="login-form">
            <form action="/login" method="POST">
                <div class="form-group">
                    <label for="username">Username</label>
                    <input type="text" id="username" name="username" required autofocus>
                </div>
                <div class="form-group">
                    <label for="password">Password</label>
                    <input type="password" id="password" name="password" required>
                </div>
                <button type="submit">Login</button>
            </form>
    "#,
    );

    if let Some(err) = error {
        html.push_str(&format!(r#"<div class="error-message">{}</div>"#, err));
    }

    html.push_str(
        r#"
        </div>
        <div class="login-info">
            <p>Default admin credentials:</p>
            <code>Username: admin<br>Password: admin123</code>
        </div>
    </div>
    "#,
    );

    html.push_str(&base_end());
    html
}

pub fn render_home(
    username: Option<&str>,
    trending: &[SearchResult],
    popular_tv: &[SearchResult],
    trending_searches: &[SearchResult],
) -> String {
    let mut html = String::new();

    html.push_str(&base_start("RustStream", username));

    html.push_str(
        r#"
    <div class="home-page">
        <h1>Welcome to RustStream</h1>
        <p>Your favorite movies and TV shows, streamed locally.</p>
        
        <section class="search-suggestions">
            <h2>Trending Searches</h2>
            <div class="suggestion-tags">
"#,
    );

    for item in trending_searches.iter().take(10) {
        let name = item
            .title
            .as_ref()
            .or(item.name.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("Unknown");
        let link = if item.media_type == "movie" {
            format!("/movie/{}", item.id)
        } else {
            format!("/tv/{}", item.id)
        };
        html.push_str(&format!(
            r#"<a href="{}" class="suggestion-tag">{}</a>"#,
            link, name
        ));
    }

    html.push_str(
        r#"
            </div>
        </section>
        
        <section class="content-section">
            <h2>Trending Movies</h2>
            <div class="content-grid">
"#,
    );

    for movie in trending {
        let poster = movie
            .poster_path
            .as_ref()
            .map(|p| format!("https://image.tmdb.org/t/p/w342{}", p))
            .unwrap_or_else(|| "/static/placeholder.jpg".to_string());
        let title = movie
            .title
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("Unknown");
        html.push_str(&format!(
            r#"<div class="content-card"><a href="/movie/{}"><img src="{}" alt="Movie" onerror="this.src='/static/placeholder.jpg'"><div class="card-info"><h3>{}</h3><p class="rating">⭐ {:.1}</p></div></a></div>"#,
            movie.id, poster, title, movie.vote_average
        ));
    }

    html.push_str(
        r#"
            </div>
        </section>
        
        <section class="content-section">
            <h2>Popular TV Shows</h2>
            <div class="content-grid">
"#,
    );

    for show in popular_tv {
        let poster = show
            .poster_path
            .as_ref()
            .map(|p| format!("https://image.tmdb.org/t/p/w342{}", p))
            .unwrap_or_else(|| "/static/placeholder.jpg".to_string());
        let name = show.name.as_ref().map(|s| s.as_str()).unwrap_or("Unknown");
        html.push_str(&format!(
            r#"<div class="content-card"><a href="/tv/{}"><img src="{}" alt="TV Show" onerror="this.src='/static/placeholder.jpg'"><div class="card-info"><h3>{}</h3><p class="rating">⭐ {:.1}</p></div></a></div>"#,
            show.id, poster, name, show.vote_average
        ));
    }

    html.push_str(
        r#"
            </div>
        </section>
    </div>
"#,
    );

    html.push_str(&base_end());
    html
}

pub fn render_search(
    username: Option<&str>,
    query: &str,
    results: &[SearchResult],
    genres: &[Genre],
) -> String {
    let mut html = String::new();

    html.push_str(&base_start("Search - RustStream", username));

    html.push_str(
        r#"
    <div class="search-page">
        <h1>Search Movies & TV Shows</h1>
        <form class="search-box" action="/search" method="get">
            <input type="text" name="q" placeholder="Search for movies, TV shows..." value=""#,
    );
    html.push_str(query);
    html.push_str(
        r#"" autofocus>
            <button type="submit">Search</button>
        </form>
        
        <details class="search-filters">
            <summary>Filters</summary>
            <div class="filter-grid">
                <div class="filter-group">
                    <label for="genre">Genre</label>
                    <select id="genre" name="genre">
                        <option value="">All Genres</option>
"#,
    );

    for genre in genres {
        html.push_str(&format!(
            r#"<option value="{}">{}</option>"#,
            genre.name.to_lowercase(),
            genre.name
        ));
    }

    html.push_str(
        r#"
                    </select>
                </div>
                <div class="filter-group">
                    <label for="year">Year</label>
                    <input type="number" id="year" name="year" placeholder="e.g. 2023" min="1900" max="2099">
                </div>
                <div class="filter-group">
                    <label for="min_rating">Min Rating</label>
                    <select id="min_rating" name="min_rating">
                        <option value="">Any</option>
                        <option value="9">9+</option>
                        <option value="8">8+</option>
                        <option value="7">7+</option>
                        <option value="6">6+</option>
                        <option value="5">5+</option>
                    </select>
                </div>
                <div class="filter-group">
                    <label for="sort_by">Sort By</label>
                    <select id="sort_by" name="sort_by">
                        <option value="popularity.desc">Popularity</option>
                        <option value="vote_average.desc">Top Rated</option>
                        <option value="release_date.desc">Newest</option>
                        <option value="revenue.desc">Highest Grossing</option>
                    </select>
                </div>
            </div>
        </details>
"#,
    );

    if !query.is_empty() || results.is_empty() == false {
        if results.is_empty() {
            html.push_str(&format!(
                r#"<div class="no-results">No results found</div>"#,
            ));
        } else {
            html.push_str(r#"<div class="content-grid">"#);
            for item in results {
                let poster = item
                    .poster_path
                    .as_ref()
                    .map(|p| format!("https://image.tmdb.org/t/p/w342{}", p))
                    .unwrap_or_else(|| "/static/placeholder.jpg".to_string());
                let name = item
                    .title
                    .as_ref()
                    .or(item.name.as_ref())
                    .map(|s| s.as_str())
                    .unwrap_or("Unknown");
                let link = if item.media_type == "movie" {
                    format!("/movie/{}", item.id)
                } else {
                    format!("/tv/{}", item.id)
                };
                let media_label = if item.media_type == "movie" {
                    "Movie"
                } else {
                    "TV Show"
                };
                html.push_str(&format!(
                    r#"<div class="content-card"><a href="{}"><img src="{}" alt="Content" onerror="this.src='/static/placeholder.jpg'"><div class="card-info"><h3>{}</h3><p class="rating">⭐ {:.1}</p><span class="media-type">{}</span></div></a></div>"#,
                    link, poster, name, item.vote_average, media_label
                ));
            }
            html.push_str("</div>");
        }
    }

    html.push_str("</div>");
    html.push_str(&base_end());
    html
}

pub fn render_movie_detail(username: Option<&str>, movie: &MovieDetail) -> String {
    let mut html = String::new();

    html.push_str(&base_start(&movie.title, username));

    let backdrop = movie
        .backdrop_path
        .as_ref()
        .map(|p| format!("https://image.tmdb.org/t/p/original{}", p))
        .unwrap_or_default();
    let poster = movie
        .poster_path
        .as_ref()
        .map(|p| format!("https://image.tmdb.org/t/p/w500{}", p))
        .unwrap_or_else(|| "/static/placeholder.jpg".to_string());
    let year = movie
        .release_date
        .as_ref()
        .and_then(|d| d.split('-').next())
        .unwrap_or("");
    let runtime = movie
        .runtime
        .map(|r| format!("{}h {}m", r / 60, r % 60))
        .unwrap_or_default();
    let genres: Vec<_> = movie.genres.iter().map(|g| g.name.clone()).collect();
    let genres_str = genres.join(", ");
    let overview = movie
        .overview
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("No overview available.");

    html.push_str(&format!(
        r#"<div class="detail-page"><div class="detail-hero" style="background-image: linear-gradient(rgba(0,0,0,0.7), rgba(0,0,0,0.9)), url({});"><div class="detail-content"><img class="detail-poster" src="{}" alt="{}" onerror="this.src='/static/placeholder.jpg'"><div class="detail-info"><h1>{}</h1><div class="meta"><span class="rating">⭐ {:.1} ({} votes)</span><span class="year">{}</span><span class="runtime">{}</span></div><p class="genres">{}</p><p class="overview">{}</p><div class="actions"><a href="/player/movie/{}" class="play-button">▶ Watch Now</a></div></div></div></div>"#,
        backdrop, poster, movie.title, movie.title, movie.vote_average, movie.vote_count, year, runtime, genres_str, overview, movie.id
    ));

    if let Some(ref credits) = movie.credits {
        html.push_str(r#"<section class="cast-section"><h2>Cast</h2><div class="cast-grid">"#);
        for member in &credits.cast {
            let profile = member
                .profile_path
                .as_ref()
                .map(|p| format!("https://image.tmdb.org/t/p/w185{}", p))
                .unwrap_or_else(|| "/static/placeholder-avatar.jpg".to_string());
            html.push_str(&format!(
                r#"<div class="cast-member"><img src="{}" alt="{}" onerror="this.src='/static/placeholder-avatar.jpg'"><h4>{}</h4><p>{}</p></div>"#,
                profile, member.name, member.name, member.character
            ));
        }
        html.push_str("</div></section>");
    }

    if let Some(ref similar) = movie.similar {
        html.push_str(
            r#"<section class="similar-section"><h2>Similar Movies</h2><div class="content-grid">"#,
        );
        for item in &similar.results {
            let poster = item
                .poster_path
                .as_ref()
                .map(|p| format!("https://image.tmdb.org/t/p/w342{}", p))
                .unwrap_or_else(|| "/static/placeholder.jpg".to_string());
            let title = item.title.as_ref().map(|s| s.as_str()).unwrap_or("Unknown");
            html.push_str(&format!(
                r#"<div class="content-card"><a href="/movie/{}"><img src="{}" alt="Movie" onerror="this.src='/static/placeholder.jpg'"><div class="card-info"><h3>{}</h3></div></a></div>"#,
                item.id, poster, title
            ));
        }
        html.push_str("</div></section>");
    }

    html.push_str("</div>");
    html.push_str(&base_end());
    html
}

pub fn render_tv_detail(username: Option<&str>, show: &TvShowDetail) -> String {
    let mut html = String::new();

    html.push_str(&base_start(&show.name, username));

    let backdrop = show
        .backdrop_path
        .as_ref()
        .map(|p| format!("https://image.tmdb.org/t/p/original{}", p))
        .unwrap_or_default();
    let poster = show
        .poster_path
        .as_ref()
        .map(|p| format!("https://image.tmdb.org/t/p/w500{}", p))
        .unwrap_or_else(|| "/static/placeholder.jpg".to_string());
    let year = show
        .first_air_date
        .as_ref()
        .and_then(|d| d.split('-').next())
        .unwrap_or("");
    let seasons = show
        .number_of_seasons
        .map(|s| format!("{} seasons", s))
        .unwrap_or_default();
    let genres: Vec<_> = show.genres.iter().map(|g| g.name.clone()).collect();
    let genres_str = genres.join(", ");
    let overview = show
        .overview
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("No overview available.");

    html.push_str(&format!(
        r#"<div class="detail-page"><div class="detail-hero" style="background-image: linear-gradient(rgba(0,0,0,0.7), rgba(0,0,0,0.9)), url({});"><div class="detail-content"><img class="detail-poster" src="{}" alt="{}" onerror="this.src='/static/placeholder.jpg'"><div class="detail-info"><h1>{}</h1><div class="meta"><span class="rating">⭐ {:.1} ({} votes)</span><span class="year">{}</span><span class="seasons">{}</span></div><p class="genres">{}</p><p class="overview">{}</p></div></div></div>"#,
        backdrop, poster, show.name, show.name, show.vote_average, show.vote_count, year, seasons, genres_str, overview
    ));

    if !show.seasons.is_empty() {
        html.push_str(
            r#"<section class="seasons-section"><h2>Seasons</h2><div class="season-list">"#,
        );
        for season in &show.seasons {
            if season.season_number > 0 {
                html.push_str(&format!(
                    r#"<div class="season-item"><h3>{}</h3><p>{} episodes</p><a href="/player/tv/{}?season={}&episode=1" class="play-button-small">▶ Play</a></div>"#,
                    season.name, season.episode_count, show.id, season.season_number
                ));
            }
        }
        html.push_str("</div></section>");
    }

    if let Some(ref credits) = show.credits {
        html.push_str(r#"<section class="cast-section"><h2>Cast</h2><div class="cast-grid">"#);
        for member in &credits.cast {
            let profile = member
                .profile_path
                .as_ref()
                .map(|p| format!("https://image.tmdb.org/t/p/w185{}", p))
                .unwrap_or_else(|| "/static/placeholder-avatar.jpg".to_string());
            html.push_str(&format!(
                r#"<div class="cast-member"><img src="{}" alt="{}" onerror="this.src='/static/placeholder-avatar.jpg'"><h4>{}</h4><p>{}</p></div>"#,
                profile, member.name, member.name, member.character
            ));
        }
        html.push_str("</div></section>");
    }

    if let Some(ref similar) = show.similar {
        html.push_str(
            r#"<section class="similar-section"><h2>Similar Shows</h2><div class="content-grid">"#,
        );
        for item in &similar.results {
            let poster = item
                .poster_path
                .as_ref()
                .map(|p| format!("https://image.tmdb.org/t/p/w342{}", p))
                .unwrap_or_else(|| "/static/placeholder.jpg".to_string());
            let name = item.name.as_ref().map(|s| s.as_str()).unwrap_or("Unknown");
            html.push_str(&format!(
                r#"<div class="content-card"><a href="/tv/{}"><img src="{}" alt="Show" onerror="this.src='/static/placeholder.jpg'"><div class="card-info"><h3>{}</h3></div></a></div>"#,
                item.id, poster, name
            ));
        }
        html.push_str("</div></section>");
    }

    html.push_str("</div>");
    html.push_str(&base_end());
    html
}

pub fn render_player(
    username: Option<&str>,
    title: &str,
    media_type: &str,
    id: i64,
    poster_path: Option<&str>,
    streams: &[StreamSource],
    is_admin: bool,
) -> String {
    let mut html = String::new();

    html.push_str(&base_start(&format!("{} - RustStream", title), username));

    let back_link = if media_type == "movie" {
        format!("/movie/{}", id)
    } else {
        format!("/tv/{}", id)
    };

    let poster_url = poster_path.map(|p| format!("https://image.tmdb.org/t/p/w500{}", p));

    html.push_str(&format!(
        r#"<div class="player-page" data-media-id="{}" data-media-type="{}"><div class="player-header"><a href="{}" class="back-button">← Back</a><h1>{}</h1></div><div class="player-container">"#,
        id, media_type, back_link, title
    ));

    if streams.is_empty() {
        html.push_str(
            r#"<div class="no-streams"><p>No streams available for this title.</p></div>"#,
        );
    } else {
        // Use iframe for vidking embed
        // Admin users get ad-blocking features
        let sandbox_attr = if is_admin {
            r#"sandbox="allow-scripts allow-same-origin allow-fullscreen allow-presentation""#
        } else {
            ""
        };

        html.push_str(&format!(
            r#"<iframe id="videoPlayer" class="video-player" src="{}" frameborder="0" allowfullscreen scrolling="no" allow="autoplay; fullscreen" {}></iframe>"#,
            streams[0].id, sandbox_attr
        ));

        if streams.len() > 1 {
            html.push_str(r#"<div class="stream-selector"><h3>Select Source:</h3>"#);
            for stream in streams {
                let quality = stream
                    .quality
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or("Unknown");
                html.push_str(&format!(
                    r#"<button>{} - {}</button>"#,
                    stream.name, quality
                ));
            }
            html.push_str("</div>");
        }
    }

    html.push_str("</div></div>");

    // Add progress tracking script
    let ad_blocking_script = if is_admin {
        r#"
    <script>
    // ADMIN: Ad blocking and popup prevention
    (function() {
        // Block popups and new windows
        window.open = function() { 
            console.log('Popup blocked by admin protection');
            return null; 
        };
        
        // Block beforeunload prompts
        window.onbeforeunload = null;
        
        // Intercept click events on the iframe
        document.addEventListener('click', function(e) {
            // Check if click is targeting the iframe
            if (e.target.tagName === 'IFRAME') {
                e.preventDefault();
                e.stopPropagation();
                console.log('Admin mode: Click intercepted');
            }
        }, true);
        
        // Block window.focus changes
        var originalFocus = window.focus;
        window.focus = function() {
            console.log('Focus change blocked');
        };
    })();
    </script>
    "#
    } else {
        ""
    };

    html.push_str(ad_blocking_script);

    let tmdb_id = id;
    let media_type = media_type;
    let progress_tracking_script = format!(
        r#"
    <script>
    const TMDB_ID = {};
    const MEDIA_TYPE = "{}";
    const TITLE = "{}";
    const POSTER_PATH = "{}";
    
    window.addEventListener("message", function(event) {{
        try {{
            var data = JSON.parse(event.data);
            console.log("Player event:", data);
            
            if (data.type === "PLAYER_EVENT") {{
                var progressData = {{
                    tmdb_id: TMDB_ID,
                    media_type: MEDIA_TYPE,
                    progress: data.data.progress || 0,
                    current_time: data.data.currentTime || 0,
                    duration: data.data.duration || 0,
                    season: data.data.season || null,
                    episode: data.data.episode || null,
                    title: TITLE,
                    poster_path: POSTER_PATH || null,
                    episode_title: null,
                    completed: data.data.event === "ended"
                }};
                
                if (data.data.event === "ended") {{
                    progressData.completed = true;
                }}
                
                fetch('/api/progress', {{
                    method: 'POST',
                    headers: {{
                        'Content-Type': 'application/json'
                    }},
                    body: JSON.stringify(progressData)
                }}).catch(e => console.log('Progress save failed:', e));
            }}
        }} catch(e) {{
            // Not a JSON message, ignore
        }}
    }});
    </script>
    "#,
        id,
        media_type,
        title,
        poster_url
            .as_ref()
            .map(|p| p.replace("https://image.tmdb.org/t/p/w500", ""))
            .unwrap_or_default()
    );

    html.push_str(&progress_tracking_script);

    // Add progress tracking script
    html.push_str(
        r#"
    <script>
    // Progress tracking for vidking player
    window.addEventListener("message", function(event) {
        try {
            var data = JSON.parse(event.data);
            console.log("Player event:", data);
            
            // Save progress to localStorage
            if (data.type === "PLAYER_EVENT") {
                var key = "progress_" + data.data.id + "_" + (data.data.mediaType || "movie");
                var progress = {
                    currentTime: data.data.currentTime,
                    duration: data.data.duration,
                    progress: data.data.progress,
                    timestamp: Date.now()
                };
                localStorage.setItem(key, JSON.stringify(progress));
                
                // Handle different events
                switch(data.data.event) {
                    case "ended":
                        console.log("Video ended");
                        break;
                    case "play":
                        console.log("Video started playing");
                        break;
                    case "pause":
                        console.log("Video paused at:", data.data.currentTime);
                        break;
                    case "seeked":
                        console.log("Seeked to:", data.data.currentTime);
                        break;
                }
            }
        } catch(e) {
            // Not a JSON message, ignore
        }
    });
    </script>
    "#,
    );

    html.push_str(&base_end());
    html
}

pub fn render_watch_history(
    username: Option<&str>,
    history: &[crate::auth::WatchHistoryItem],
) -> String {
    let mut html = String::new();

    html.push_str(&base_start("Watch History - RustStream", username));

    html.push_str(
        r#"
    <div class="history-page">
        <h1>Your Watch History</h1>
"#,
    );

    if history.is_empty() {
        html.push_str(
            r#"<div class="no-results">
            <p>You haven't watched anything yet.</p>
            <a href="/search" class="play-button">Browse Movies & TV Shows</a>
        </div>"#,
        );
    } else {
        html.push_str(r#"<div class="content-grid">"#);

        for item in history {
            let poster = item
                .poster_path
                .as_ref()
                .map(|p| format!("https://image.tmdb.org/t/p/w342{}", p))
                .unwrap_or_else(|| "/static/placeholder.jpg".to_string());

            let link = if item.media_type == "movie" {
                format!("/movie/{}", item.tmdb_id)
            } else if item.season_number.is_some() && item.episode_number.is_some() {
                format!(
                    "/player/tv/{}?season={}&episode={}",
                    item.tmdb_id,
                    item.season_number.unwrap(),
                    item.episode_number.unwrap()
                )
            } else {
                format!("/tv/{}", item.tmdb_id)
            };

            let label = if item.media_type == "movie" {
                "Movie"
            } else if let (Some(season), Some(episode)) = (item.season_number, item.episode_number)
            {
                &format!("S{}E{}", season, episode)
            } else {
                "TV Show"
            };

            let progress_bar = if item.completed {
                r#"<div class="progress-bar"><div class="progress-bar-fill" style="width: 100%;"></div></div>
                   <span class="completed-badge">✓ Completed</span>"#.to_string()
            } else if item.progress_seconds > 0 {
                let pct = std::cmp::min(item.progress_seconds / 60, 100);
                format!(
                    r#"<div class="progress-bar"><div class="progress-bar-fill" style="width: {}%;"></div></div>
                   <span class="progress-time">{} min watched</span>"#,
                    pct,
                    item.progress_seconds / 60
                )
            } else {
                String::new()
            };

            html.push_str(&format!(
                r#"<div class="content-card"><a href="{}"><img src="{}" alt="{}" onerror="this.src='/static/placeholder.jpg'"><div class="card-info"><h3>{}</h3><p class="rating">{}</p>{}</div></a></div>"#,
                link, poster, item.title, item.title, label, progress_bar
            ));
        }
        html.push_str("</div>");
    }

    html.push_str("</div>");
    html.push_str(&base_end());
    html
}

fn base_start(title: &str, username: Option<&str>) -> String {
    let nav_links = if let Some(user) = username {
        format!(
            r#"<a href="/">Home</a>
            <a href="/search">Search</a>
            <a href="/history">History</a>
            <span class="user-info">👤 {}</span>
            <a href="/logout" class="logout-btn">Logout</a>"#,
            user
        )
    } else {
        String::from(
            r#"<a href="/">Home</a>
            <a href="/search">Search</a>
            <a href="/login">Login</a>"#,
        )
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    <nav class="navbar">
        <div class="nav-brand">
            <a href="/">RustStream</a>
        </div>
        <div class="nav-links">
            {}
        </div>
    </nav>
    <main>"#,
        title, nav_links
    )
}

fn base_end() -> String {
    String::from(r#"</main></body></html>"#)
}
