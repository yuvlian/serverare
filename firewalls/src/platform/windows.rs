use std::fmt::Write;
use std::process::Command;

use crate::db::Db;
use crate::manager::FirewallManager;
use crate::manager::get_rule_name;
use common::error::FirewallError;
use common::models::Relay;

pub struct WindowsFirewallManager;

fn netsh_ok(output: &std::process::Output, rule_name: &str, op: &str) -> Result<(), FirewallError> {
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let is_permission_error = stdout.contains("elevation")
            || stderr.contains("elevation")
            || stdout.contains("administrator")
            || stderr.contains("administrator")
            || stdout.contains("privilege")
            || stderr.contains("privilege")
            || stdout.contains("Access is denied")
            || stderr.contains("Access is denied");

        if is_permission_error {
            return Err(FirewallError::Command(format!(
                "netsh {op} '{rule_name}' failed (stdout: {stdout}, stderr: {stderr})"
            )));
        }
    }
    Ok(())
}

impl FirewallManager for WindowsFirewallManager {
    fn block_server(
        &self,
        name: &str,
        ips: &[Relay],
        description: &str,
    ) -> Result<(), FirewallError> {
        let rule_name = get_rule_name(name);
        let _ = self.unblock_server(name);

        let mut ip_list = String::with_capacity(ips.len() * 16);
        for (i, ip) in ips.iter().enumerate() {
            if i > 0 {
                ip_list.push(',');
            }
            write!(&mut ip_list, "{}", ip.ipv4).ok();
        }
        let output = Command::new("netsh")
            .args([
                "advfirewall",
                "firewall",
                "add",
                "rule",
                &format!("name={rule_name}"),
                "dir=out",
                "action=block",
                &format!("remoteip={ip_list}"),
                "enable=yes",
                &format!("description={description}"),
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(FirewallError::Command(format!(
                "netsh add rule '{rule_name}' failed (ips: {ip_list}, stdout: {stdout}, stderr: {stderr})"
            )));
        }

        Db::block_server(name, ips)
    }

    fn unblock_server(&self, name: &str) -> Result<(), FirewallError> {
        let rule_name = get_rule_name(name);
        let output = Command::new("netsh")
            .args([
                "advfirewall",
                "firewall",
                "delete",
                "rule",
                &format!("name={rule_name}"),
            ])
            .output()?;

        netsh_ok(&output, &rule_name, "delete rule")?;
        Db::unblock_server(name)
    }

    fn is_server_blocked(&self, name: &str) -> bool {
        Db::is_blocked(name)
    }

    fn block_servers(&self, servers: &[(String, Vec<Relay>, String)]) -> Result<(), FirewallError> {
        let mut db_updates = Vec::with_capacity(servers.len());
        for (name, ips, description) in servers {
            let rule_name = get_rule_name(name);
            let _ = self.unblock_server(name);

            let mut ip_list = String::with_capacity(ips.len() * 16);
            for (i, ip) in ips.iter().enumerate() {
                if i > 0 {
                    ip_list.push(',');
                }
                write!(&mut ip_list, "{}", ip.ipv4).ok();
            }
            let output = Command::new("netsh")
                .args([
                    "advfirewall",
                    "firewall",
                    "add",
                    "rule",
                    &format!("name={rule_name}"),
                    "dir=out",
                    "action=block",
                    &format!("remoteip={ip_list}"),
                    "enable=yes",
                    &format!("description={description}"),
                ])
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                return Err(FirewallError::Command(format!(
                    "netsh add rule '{rule_name}' failed (ips: {ip_list}, stdout: {stdout}, stderr: {stderr})"
                )));
            }
            db_updates.push((name.clone(), ips.clone()));
        }

        Db::block_servers(&db_updates)
    }

    fn unblock_servers(&self, names: &[String]) -> Result<(), FirewallError> {
        for name in names {
            let rule_name = get_rule_name(name);
            let output = Command::new("netsh")
                .args([
                    "advfirewall",
                    "firewall",
                    "delete",
                    "rule",
                    &format!("name={rule_name}"),
                ])
                .output()?;

            netsh_ok(&output, &rule_name, "delete rule")?;
        }
        Db::unblock_servers(names)
    }
}
