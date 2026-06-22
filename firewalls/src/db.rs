use std::collections::HashMap;
use std::fs;
use std::net::IpAddr;
use std::sync::LazyLock;
use std::sync::Mutex;

use common::error::FirewallError;
use common::models::Relay;

const DB_PATH: &str = "./serverare_blocklist.json";

pub type DbMap = HashMap<String, Vec<IpAddr>>;

pub struct Db;

static DATA: LazyLock<Mutex<DbMap>> = LazyLock::new(|| {
    let map = match fs::read_to_string(DB_PATH) {
        Ok(s) => serde_json::from_str::<DbMap>(&s).unwrap_or_default(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            fs::write(DB_PATH, "{}").unwrap();
            HashMap::new()
        }
        Err(e) => {
            panic!("failed deserializing {}: {}", DB_PATH, e);
        }
    };
    Mutex::new(map)
});

impl Db {
    fn save_locked(guard: &DbMap) -> Result<(), FirewallError> {
        let json = serde_json::to_string_pretty(guard)
            .map_err(|e| FirewallError::Command(format!("Failed to serialize DB: {e}")))?;
        fs::write(DB_PATH, json)?;
        Ok(())
    }

    fn lock() -> std::sync::MutexGuard<'static, DbMap> {
        DATA.lock().unwrap()
    }

    pub fn block_server(name: &str, ips: &[Relay]) -> Result<(), FirewallError> {
        let mut guard = Self::lock();
        let ips: Vec<IpAddr> = ips.iter().map(|r| r.ipv4).collect();
        guard.insert(name.to_string(), ips);
        Self::save_locked(&guard)
    }

    pub fn unblock_server(name: &str) -> Result<(), FirewallError> {
        let mut guard = Self::lock();
        guard.remove(name);
        Self::save_locked(&guard)
    }

    pub fn is_blocked(name: &str) -> bool {
        Self::lock().contains_key(name)
    }

    pub fn get_ips(name: &str) -> Option<Vec<IpAddr>> {
        Self::lock().get(name).cloned()
    }

    pub fn block_servers(updates: &[(String, Vec<Relay>)]) -> Result<(), FirewallError> {
        let mut guard = Self::lock();
        for (name, ips) in updates {
            let ips: Vec<IpAddr> = ips.iter().map(|r| r.ipv4).collect();
            guard.insert(name.clone(), ips);
        }
        Self::save_locked(&guard)
    }

    pub fn unblock_servers(names: &[String]) -> Result<(), FirewallError> {
        let mut guard = Self::lock();
        for name in names {
            guard.remove(name);
        }
        Self::save_locked(&guard)
    }
}
