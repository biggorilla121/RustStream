use std::io;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

const DEFAULT_DATABASE_URL: &str = "sqlite://./streaming.db";
const DEFAULT_PORT: &str = "3000";

#[derive(Debug, Clone)]
pub struct OnboardingConfig {
    pub tmdb_api_key: String,
    pub database_url: String,
    pub port: u16,
}

pub fn maybe_run_onboarding() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    if std::env::var("TMDB_API_KEY").is_ok() {
        return Ok(());
    }

    let config = run_onboarding()?;
    write_env_file(&config)?;

    std::env::set_var("TMDB_API_KEY", &config.tmdb_api_key);
    std::env::set_var("DATABASE_URL", &config.database_url);
    std::env::set_var("PORT", config.port.to_string());

    Ok(())
}

fn run_onboarding() -> anyhow::Result<OnboardingConfig> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = OnboardingState::new();
    let result = (|| -> anyhow::Result<OnboardingConfig> {
        loop {
            terminal.draw(|f| render_onboarding(f, &state))?;

            if let Event::Key(key) = event::read()? {
                if handle_key_event(&mut state, key) {
                    return state.build_config();
                }

                if state.exit_requested {
                    return Err(anyhow::anyhow!("Onboarding cancelled"));
                }
            }
        }
    })();

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn write_env_file(config: &OnboardingConfig) -> anyhow::Result<()> {
    let mut contents = String::new();
    contents.push_str("# TMDB API Key (v4 auth read token)\n");
    contents.push_str("# Get it from: https://www.themoviedb.org/settings/api\n");
    contents.push_str(&format!("TMDB_API_KEY={}\n\n", config.tmdb_api_key));

    contents.push_str("# Database URL (SQLite)\n");
    contents.push_str(&format!("DATABASE_URL={}\n\n", config.database_url));

    contents.push_str("# Server port (optional, defaults to 3000)\n");
    contents.push_str(&format!("PORT={}\n", config.port));

    std::fs::write(".env", contents)?;
    Ok(())
}

#[derive(Debug, Clone)]
struct OnboardingState {
    step: usize,
    tmdb_api_key: String,
    database_url: String,
    port: String,
    exit_requested: bool,
}

impl OnboardingState {
    fn new() -> Self {
        Self {
            step: 0,
            tmdb_api_key: String::new(),
            database_url: DEFAULT_DATABASE_URL.to_string(),
            port: DEFAULT_PORT.to_string(),
            exit_requested: false,
        }
    }

    fn is_complete(&self) -> bool {
        !self.tmdb_api_key.trim().is_empty()
    }

    fn build_config(&self) -> anyhow::Result<OnboardingConfig> {
        if !self.is_complete() {
            return Err(anyhow::anyhow!("TMDB API key is required"));
        }

        let port_str = self.port.trim();
        let port: u16 = if port_str.is_empty() {
            DEFAULT_PORT.parse().unwrap_or(3000)
        } else {
            port_str
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid port"))?
        };

        Ok(OnboardingConfig {
            tmdb_api_key: self.tmdb_api_key.trim().to_string(),
            database_url: {
                let url = self.database_url.trim();
                if url.is_empty() {
                    DEFAULT_DATABASE_URL.to_string()
                } else {
                    url.to_string()
                }
            },
            port,
        })
    }
}

fn handle_key_event(state: &mut OnboardingState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            state.exit_requested = true;
            false
        }
        KeyCode::Char('q') if key.modifiers.is_empty() => {
            state.exit_requested = true;
            false
        }
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.is_complete()
        }
        KeyCode::Enter => {
            if state.step < 2 {
                state.step += 1;
                false
            } else {
                state.is_complete()
            }
        }
        KeyCode::Tab | KeyCode::Down => {
            if state.step < 2 {
                state.step += 1;
            }
            false
        }
        KeyCode::Up => {
            if state.step > 0 {
                state.step -= 1;
            }
            false
        }
        KeyCode::Backspace => {
            current_field(state).pop();
            false
        }
        KeyCode::Char(c) => {
            if state.step == 2 {
                if c.is_ascii_digit() {
                    current_field(state).push(c);
                }
            } else {
                current_field(state).push(c);
            }
            false
        }
        _ => false,
    }
}

fn current_field(state: &mut OnboardingState) -> &mut String {
    match state.step {
        0 => &mut state.tmdb_api_key,
        1 => &mut state.database_url,
        _ => &mut state.port,
    }
}

fn render_onboarding(f: &mut ratatui::Frame, state: &OnboardingState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Length(9),
            Constraint::Length(3),
        ])
        .split(f.area());

    let title = Paragraph::new(Line::from(vec![
        Span::styled("RustStream Onboarding", Style::default().add_modifier(Modifier::BOLD)),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));

    f.render_widget(title, layout[0]);

    let intro = Paragraph::new(vec![
        Line::from("Welcome. This setup runs once and writes a .env file."),
        Line::from("You can change values later in .env."),
        Line::from(""),
        Line::from("Controls: Enter/Tab next, Up/Down previous, Ctrl+S save, Esc quit."),
    ])
    .block(Block::default().borders(Borders::NONE));

    f.render_widget(intro, layout[1]);

    let fields = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(layout[2]);

    render_field(
        f,
        fields[0],
        "TMDB API Key (required)",
        &state.tmdb_api_key,
        state.step == 0,
    );

    render_field(
        f,
        fields[1],
        "Database URL",
        &state.database_url,
        state.step == 1,
    );

    render_field(
        f,
        fields[2],
        "Port",
        &state.port,
        state.step == 2,
    );

    let status_style = if state.is_complete() {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Red)
    };

    let status = Paragraph::new(Line::from(vec![
        Span::styled(
            if state.is_complete() {
                "Ready to save"
            } else {
                "TMDB API key required"
            },
            status_style,
        ),
    ]))
    .block(Block::default().borders(Borders::TOP));

    f.render_widget(status, layout[3]);
}

fn render_field(
    f: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    label: &str,
    value: &str,
    active: bool,
) {
    let title = if active {
        format!("{} (editing)", label)
    } else {
        label.to_string()
    };

    let style = if active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let block = Block::default().title(title).borders(Borders::ALL).border_style(style);
    let paragraph = Paragraph::new(value).block(block).style(style);
    f.render_widget(paragraph, area);
}
