mod app;
mod draw;
mod events;

use std::io;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::time::Duration;

use app::ActivePane;
use app::AppState;
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEventKind;
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use events::AppEvent;
use firewalls::new_firewall_manager;
use firewalls::platform::is_admin;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use serverare::cache;
use serverare::fetch::load_game_definitions;

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    if !is_admin() {
        return Err("This application requires admin/root.".into());
    }

    let games = load_game_definitions()?;

    if games.is_empty() {
        return Err("No game definitions found.".into());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (event_tx, event_rx) = mpsc::channel::<AppEvent>();
    let mut app_state = AppState::new(games);
    app_state.change_game(event_tx.clone());

    loop {
        terminal
            .draw(|f| draw::draw(f, &mut app_state))
            .map_err(|e| format!("Draw error: {e}"))?;

        while let Ok(event) = event_rx.try_recv() {
            handle_event(&mut app_state, event, &event_tx);
        }

        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
            && handle_key(&mut app_state, key.code, &event_tx)
        {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn handle_event(state: &mut AppState, event: AppEvent, tx: &Sender<AppEvent>) {
    match event {
        AppEvent::FetchSuccess { servers } => {
            state.servers = servers;
            state.selected_server_idx = 0;
            state.servers_table_state.select(Some(0));
            state.loading = false;
            state.error_message = None;
            state.status_message =
                "Servers loaded. Press R to refresh pings or Up/Down to navigate.".to_string();
            state.trigger_pings(tx.clone());
        }
        AppEvent::FetchError(e) => {
            state.loading = false;
            state.error_message = Some(e);
        }
        AppEvent::PingUpdate {
            index,
            ping,
            status_text,
        } => {
            if index < state.servers.len() {
                state.servers[index].ping = ping;
                state.servers[index].status_text = status_text;
            }
            if state.servers.iter().all(|s| s.status_text != "Checking...") {
                state.status_message = "Pings updated.".to_string();
            }
        }
        AppEvent::FirewallSuccess => {
            state.loading = false;
            state.error_message = None;
            state.status_message = "Firewall updated successfully.".to_string();
            let firewall = new_firewall_manager();
            cache::update_blocked_status(&mut state.servers, &firewall);
        }
        AppEvent::FirewallError(e) => {
            state.loading = false;
            state.error_message = Some(e);
        }
    }
}

fn handle_key(state: &mut AppState, key: KeyCode, tx: &Sender<AppEvent>) -> bool {
    match key {
        KeyCode::Char('q') | KeyCode::Esc => true,
        KeyCode::Left | KeyCode::Right => {
            state.active_pane = match state.active_pane {
                ActivePane::Games => ActivePane::Servers,
                ActivePane::Servers => ActivePane::Games,
            };
            false
        }
        KeyCode::Up => {
            match state.active_pane {
                ActivePane::Games => {
                    if state.selected_game_idx > 0 {
                        state.selected_game_idx -= 1;
                        state.games_list_state.select(Some(state.selected_game_idx));
                        state.change_game(tx.clone());
                    }
                }
                ActivePane::Servers => {
                    if state.selected_server_idx > 0 {
                        state.selected_server_idx -= 1;
                        state
                            .servers_table_state
                            .select(Some(state.selected_server_idx));
                    }
                }
            }
            false
        }
        KeyCode::Down => {
            match state.active_pane {
                ActivePane::Games => {
                    if state.selected_game_idx + 1 < state.games.len() {
                        state.selected_game_idx += 1;
                        state.games_list_state.select(Some(state.selected_game_idx));
                        state.change_game(tx.clone());
                    }
                }
                ActivePane::Servers => {
                    if state.selected_server_idx + 1 < state.servers.len() {
                        state.selected_server_idx += 1;
                        state
                            .servers_table_state
                            .select(Some(state.selected_server_idx));
                    }
                }
            }
            false
        }
        KeyCode::Char(' ') => {
            if state.active_pane == ActivePane::Servers && !state.loading {
                state.toggle_selected_server(tx.clone());
            }
            false
        }
        KeyCode::Char('d') => {
            if !state.loading {
                state.set_all_servers_blocked(true, tx.clone());
            }
            false
        }
        KeyCode::Char('e') => {
            if !state.loading {
                state.set_all_servers_blocked(false, tx.clone());
            }
            false
        }
        KeyCode::Char('r') => {
            if !state.loading {
                state.trigger_pings(tx.clone());
            }
            false
        }
        _ => false,
    }
}
