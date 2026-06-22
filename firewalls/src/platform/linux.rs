use std::fmt::Write;
use std::process::Command;

use crate::db::Db;
use crate::manager::FirewallManager;
use common::error::FirewallError;
use common::models::Relay;

pub struct LinuxFirewallManager;

fn delete_rules_for_ip(ip: &str) -> Result<(), FirewallError> {
    let output = Command::new("iptables")
        .args(["-D", "INPUT", "-s", ip, "-j", "DROP"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(FirewallError::Command(format!(
            "iptables -D failed for {ip} (stdout: {stdout}, stderr: {stderr})"
        )));
    }
    Ok(())
}

impl FirewallManager for LinuxFirewallManager {
    fn block_server(
        &self,
        name: &str,
        ips: &[Relay],
        _description: &str,
    ) -> Result<(), FirewallError> {
        let mut ip_list = String::with_capacity(ips.len() * 16);
        for (i, ip) in ips.iter().enumerate() {
            if i > 0 {
                ip_list.push(',');
            }
            write!(&mut ip_list, "{}", ip.ipv4).ok();
        }
        let output = Command::new("iptables")
            .args(["-A", "INPUT", "-s", &ip_list, "-j", "DROP"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(FirewallError::Command(format!(
                "iptables -A failed for {name} (ips: {ip_list}, stdout: {stdout}, stderr: {stderr})"
            )));
        }

        Db::block_server(name, ips)
    }

    fn unblock_server(&self, name: &str) -> Result<(), FirewallError> {
        if let Some(ips) = Db::get_ips(name) {
            for ip in &ips {
                delete_rules_for_ip(&ip.to_string())?;
            }
        }
        Db::unblock_server(name)
    }

    fn is_server_blocked(&self, name: &str) -> bool {
        Db::is_blocked(name)
    }

    fn block_servers(&self, servers: &[(String, Vec<Relay>, String)]) -> Result<(), FirewallError> {
        let mut db_updates = Vec::new();
        for (name, ips, _description) in servers {
            let mut ip_list = String::with_capacity(ips.len() * 16);
            for (i, ip) in ips.iter().enumerate() {
                if i > 0 {
                    ip_list.push(',');
                }
                write!(&mut ip_list, "{}", ip.ipv4).ok();
            }
            let output = Command::new("iptables")
                .args(["-A", "INPUT", "-s", &ip_list, "-j", "DROP"])
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                return Err(FirewallError::Command(format!(
                    "iptables -A failed for {name} (ips: {ip_list}, stdout: {stdout}, stderr: {stderr})"
                )));
            }
            db_updates.push((name.clone(), ips.clone()));
        }
        Db::block_servers(&db_updates)
    }

    fn unblock_servers(&self, names: &[String]) -> Result<(), FirewallError> {
        for name in names {
            if let Some(ips) = Db::get_ips(name) {
                for ip in &ips {
                    delete_rules_for_ip(&ip.to_string())?;
                }
            }
        }
        Db::unblock_servers(names)
    }
}
