use common::models::Server;

#[derive(Debug, Clone)]
pub enum AppEvent {
    FetchSuccess {
        servers: Vec<Server>,
    },
    FetchError(String),
    PingUpdate {
        index: usize,
        ping: Option<u32>,
        status_text: String,
    },
    FirewallSuccess,
    FirewallError(String),
}
