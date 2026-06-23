use common::models::Server;
use ping::Ping;
use std::time::Duration;

const PING_TIMEOUT: Duration = Duration::from_millis(800);
const PROBE_TIMEOUT: Duration = Duration::from_millis(2000);
const PROBE_COUNT: usize = 4;
const TTL: u32 = 64;

pub fn ping_server(server: &mut Server) {
    let mut best_relay_ip = None;
    let mut best_rtt = u32::MAX;

    for relay in &server.relays {
        if let Ok(reply) = Ping::new(relay.ipv4)
            .timeout(PING_TIMEOUT)
            .ttl(TTL)
            // .socket_type(ping::DGRAM)
            .send()
        {
            let rtt = reply.rtt.as_millis() as u32;
            if rtt < best_rtt {
                best_rtt = rtt;
                best_relay_ip = Some(relay.ipv4);
            }
        }
    }

    if let Some(ip) = best_relay_ip {
        let mut success_count = 0u32;
        let mut final_best_rtt = u32::MAX;

        for _ in 0..PROBE_COUNT {
            if let Ok(reply) = Ping::new(ip)
                .timeout(PROBE_TIMEOUT)
                .ttl(TTL)
                // .socket_type(ping::DGRAM)
                .send()
            {
                success_count += 1;
                let rtt = reply.rtt.as_millis() as u32;
                final_best_rtt = final_best_rtt.min(rtt);
            }
        }

        if success_count > 0 {
            server.ping = Some(final_best_rtt);
            server.packet_loss =
                Some(((PROBE_COUNT - success_count as usize) * 100 / PROBE_COUNT) as u32);
            server.status_text = format!("{final_best_rtt}ms");
        } else {
            handle_offline(server);
        }
    } else {
        handle_offline(server);
    }
}

fn handle_offline(server: &mut Server) {
    server.ping = None;
    server.packet_loss = Some(100);
    server.status_text = "Offline".to_string();
}
