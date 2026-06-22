use std::net::IpAddr;
use std::time::Duration;

use common::models::Server;
use ping_rs::PingOptions;
use ping_rs::send_ping;

const PING_TIMEOUT: Duration = Duration::from_millis(800);
const PROBE_TIMEOUT: Duration = Duration::from_millis(2000);
const PROBE_COUNT: usize = 4;
const PING_DATA: [u8; 8] = [0u8; 8];

pub fn ping_server(server: &mut Server) {
    let options = PingOptions {
        ttl: 64,
        dont_fragment: false,
    };

    let mut best_relay_ip: Option<IpAddr> = None;
    let mut best_rtt = u32::MAX;

    for relay in &server.relays {
        if let Ok(reply) = send_ping(&relay.ipv4, PING_TIMEOUT, &PING_DATA, Some(&options))
            && reply.rtt < best_rtt
        {
            best_rtt = reply.rtt;
            best_relay_ip = Some(relay.ipv4);
        }
    }

    if let Some(ip) = best_relay_ip {
        let mut success_count = 0u32;
        let mut final_best_rtt = u32::MAX;

        for _ in 0..PROBE_COUNT {
            if let Ok(reply) = send_ping(&ip, PROBE_TIMEOUT, &PING_DATA, Some(&options)) {
                success_count += 1;
                final_best_rtt = final_best_rtt.min(reply.rtt);
            }
        }

        if success_count > 0 {
            server.ping = Some(final_best_rtt);
            server.packet_loss =
                Some(((PROBE_COUNT - success_count as usize) * 100 / PROBE_COUNT) as u32);
            server.status_text = format!("{final_best_rtt}ms");
        } else {
            server.ping = None;
            server.packet_loss = Some(100);
            server.status_text = "Offline".to_string();
        }
    } else {
        server.ping = None;
        server.packet_loss = None;
        server.status_text = "Offline".to_string();
    }
}
