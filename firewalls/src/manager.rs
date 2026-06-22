use common::error::FirewallError;
use common::models::Relay;

const RULE_PREFIX: &str = "serverare_";

pub trait FirewallManager {
    fn block_server(
        &self,
        name: &str,
        ips: &[Relay],
        description: &str,
    ) -> Result<(), FirewallError>;
    fn unblock_server(&self, name: &str) -> Result<(), FirewallError>;
    fn is_server_blocked(&self, name: &str) -> bool;
    fn block_servers(&self, servers: &[(String, Vec<Relay>, String)]) -> Result<(), FirewallError>;
    fn unblock_servers(&self, names: &[String]) -> Result<(), FirewallError>;
}

pub fn get_rule_name(name: &str) -> String {
    let mut s = String::with_capacity(RULE_PREFIX.len() + name.len());
    s.push_str(RULE_PREFIX);
    s.extend(name.chars().filter(|c| !c.is_whitespace()));
    s
}
