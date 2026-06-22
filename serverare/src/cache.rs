use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Duration;
use std::time::Instant;

use common::models::Server;
use firewalls::manager::FirewallManager;
use parking_lot::Mutex;

const CACHE_TTL: Duration = Duration::from_secs(300);

struct CacheEntry {
    servers: Vec<Server>,
    timestamp: Instant,
}

static SERVER_CACHE: LazyLock<Mutex<HashMap<u32, CacheEntry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn get_cached_servers(app_id: u32) -> Option<Vec<Server>> {
    let cache = SERVER_CACHE.lock();
    cache.get(&app_id).and_then(|entry| {
        if entry.timestamp.elapsed() < CACHE_TTL {
            Some(entry.servers.clone())
        } else {
            None
        }
    })
}

pub fn put_cached_servers(app_id: u32, servers: Vec<Server>) {
    let mut cache = SERVER_CACHE.lock();
    cache.insert(
        app_id,
        CacheEntry {
            servers,
            timestamp: Instant::now(),
        },
    );
}

pub fn update_blocked_status(servers: &mut [Server], firewall: &impl FirewallManager) {
    for s in servers.iter_mut() {
        s.blocked = firewall.is_server_blocked(&s.description);
    }
}
