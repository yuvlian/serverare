use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;

use common::models::GameDefinition;
use common::models::Relay;
use common::models::Server;
use firewalls::FirewallManager;
use firewalls::new_firewall_manager;
use pinger::ping_server;
use ratatui::widgets::ListState;
use ratatui::widgets::TableState;
use serverare::cache::update_blocked_status;
use serverare::fetch::fetch_servers;

use crate::events::AppEvent;

#[derive(PartialEq)]
pub enum ActivePane {
    Games,
    Servers,
}

pub struct AppState {
    pub games: Vec<GameDefinition>,
    pub selected_game_idx: usize,
    pub servers: Vec<Server>,
    pub selected_server_idx: usize,
    pub active_pane: ActivePane,
    pub loading: bool,
    pub status_message: String,
    pub error_message: Option<String>,
    pub games_list_state: ListState,
    pub servers_table_state: TableState,
    ping_cancel: Arc<AtomicBool>,
}

impl AppState {
    pub fn new(games: Vec<GameDefinition>) -> Self {
        let mut games_list_state = ListState::default();
        games_list_state.select(Some(0));
        let mut servers_table_state = TableState::default();
        servers_table_state.select(Some(0));

        Self {
            games,
            selected_game_idx: 0,
            servers: Vec::new(),
            selected_server_idx: 0,
            active_pane: ActivePane::Games,
            loading: false,
            status_message: String::new(),
            error_message: None,
            games_list_state,
            servers_table_state,
            ping_cancel: Arc::new(AtomicBool::new(false)),
        }
    }

    fn cancel_pings(&mut self) {
        self.ping_cancel.store(true, Ordering::Relaxed);
        self.ping_cancel = Arc::new(AtomicBool::new(false));
    }

    pub fn change_game(&mut self, tx: Sender<AppEvent>) {
        if self.selected_game_idx >= self.games.len() {
            return;
        }
        self.loading = true;
        self.status_message = "Fetching servers from Steam SDR...".to_string();
        self.error_message = None;
        self.servers.clear();
        self.selected_server_idx = 0;
        self.servers_table_state.select(Some(0));
        self.games_list_state.select(Some(self.selected_game_idx));
        self.cancel_pings();

        let game_def = self.games[self.selected_game_idx].clone();
        std::thread::spawn(move || match fetch_servers(&game_def) {
            Ok(mut servers) => {
                let firewall = new_firewall_manager();
                update_blocked_status(&mut servers, &firewall);
                let _ = tx.send(AppEvent::FetchSuccess { servers });
            }
            Err(e) => {
                let _ = tx.send(AppEvent::FetchError(e.to_string()));
            }
        });
    }

    pub fn trigger_pings(&mut self, tx: Sender<AppEvent>) {
        if self.servers.is_empty() {
            return;
        }
        self.cancel_pings();
        let cancel = Arc::clone(&self.ping_cancel);
        self.status_message = "Pinging servers...".to_string();
        for s in &mut self.servers {
            s.status_text = "Checking...".to_string();
        }

        let servers_to_ping = self.servers.clone();
        std::thread::spawn(move || {
            std::thread::scope(|scope| {
                for (idx, mut server) in servers_to_ping.into_iter().enumerate() {
                    if cancel.load(Ordering::Relaxed) {
                        break;
                    }
                    let tx = tx.clone();
                    let cancel = Arc::clone(&cancel);
                    scope.spawn(move || {
                        ping_server(&mut server);
                        if !cancel.load(Ordering::Relaxed) {
                            let _ = tx.send(AppEvent::PingUpdate {
                                index: idx,
                                ping: server.ping,
                                status_text: server.status_text,
                            });
                        }
                    });
                }
            });
        });
    }

    pub fn toggle_selected_server(&mut self, tx: Sender<AppEvent>) {
        if self.servers.is_empty() || self.selected_server_idx >= self.servers.len() {
            return;
        }
        self.loading = true;
        self.status_message = "Updating firewall rules...".to_string();

        let server = self.servers[self.selected_server_idx].clone();
        let should_block = !server.blocked;

        std::thread::spawn(move || {
            let firewall = new_firewall_manager();
            let res = if should_block {
                firewall.block_server(
                    &server.description,
                    &server.relays,
                    &format!("serverare: Block {}", server.description),
                )
            } else {
                firewall.unblock_server(&server.description)
            };

            match res {
                Ok(_) => {
                    let _ = tx.send(AppEvent::FirewallSuccess);
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::FirewallError(e.to_string()));
                }
            }
        });
    }

    pub fn set_all_servers_blocked(&mut self, block: bool, tx: Sender<AppEvent>) {
        if self.servers.is_empty() {
            return;
        }
        self.loading = true;
        self.status_message = if block {
            "Blocking all servers...".to_string()
        } else {
            "Unblocking all servers...".to_string()
        };

        let servers = self.servers.clone();
        std::thread::spawn(move || {
            let firewall = new_firewall_manager();
            let res = if block {
                let batch: Vec<(String, Vec<Relay>, String)> = servers
                    .into_iter()
                    .map(|s| {
                        let desc = format!("serverare: Block {}", s.description);
                        (s.description, s.relays, desc)
                    })
                    .collect();
                firewall.block_servers(&batch)
            } else {
                let names: Vec<String> = servers.into_iter().map(|s| s.description).collect();
                firewall.unblock_servers(&names)
            };

            match res {
                Ok(_) => {
                    let _ = tx.send(AppEvent::FirewallSuccess);
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::FirewallError(e.to_string()));
                }
            }
        });
    }
}
