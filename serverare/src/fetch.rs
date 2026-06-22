use std::collections::HashMap;
use std::fs;
use std::sync::LazyLock;
use std::time::Duration;

use crate::cache::get_cached_servers;
use crate::cache::put_cached_servers;
use common::error::FetchError;
use common::models::GameDefinition;
use common::models::Relay;
use common::models::SdrConfig;
use common::models::Server;

const FETCH_TIMEOUT: Duration = Duration::from_secs(10);
const SERVER_DEFINITIONS_PATH: &str = "./server_definitions.json";

static HTTP_AGENT: LazyLock<ureq::Agent> = LazyLock::new(|| {
    ureq::Agent::config_builder()
        .timeout_global(Some(FETCH_TIMEOUT))
        .build()
        .into()
});

const DEFAULT_GAME_DEFINITIONS: &str = r#"[
  {
    "gameMode": "Counter Strike 2",
    "id": "cs2",
    "displayName": "CS2",
    "appId": 730,
    "keywordFilterMode": "exclude",
    "keywords": [
      "China"
    ],
    "clusterKeywords": [
      "Hong Kong",
      "Sweden",
      "India",
      "Netherlands"
    ]
  },
  {
    "gameMode": "Counter Strike 2 (Perfect World)",
    "id": "cs2_perfect_world",
    "displayName": "CS2 Perfect World",
    "appId": 730,
    "keywordFilterMode": "include",
    "keywords": [
      "China"
    ],
    "clusterKeywords": [
      "Tencent",
      "Alibaba",
      "Perfect World"
    ]
  },
  {
    "gameMode": "Deadlock",
    "id": "deadlock",
    "displayName": "Deadlock",
    "appId": 1422450,
    "keywordFilterMode": "none",
    "keywords": [],
    "clusterKeywords": [
      "China",
      "Hong Kong",
      "Sweden",
      "India",
      "Netherlands"
    ]
  },
  {
    "gameMode": "Marathon",
    "id": "marathon",
    "displayName": "Marathon",
    "appId": 3065800,
    "keywordFilterMode": "none",
    "keywords": [],
    "clusterKeywords": [
      "Hong Kong",
      "Sweden",
      "India",
      "Netherlands"
    ]
  },
  {
    "gameMode": "THE FINALS",
    "id": "the_finals",
    "displayName": "THE FINALS",
    "appId": 2073850,
    "keywordFilterMode": "none",
    "keywords": [],
    "clusterKeywords": [
      "Sweden",
      "India",
      "Washington"
    ]
  }
]"#;

pub fn load_game_definitions() -> Result<Vec<GameDefinition>, FetchError> {
    let raw = match fs::read_to_string(SERVER_DEFINITIONS_PATH) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            fs::write(SERVER_DEFINITIONS_PATH, DEFAULT_GAME_DEFINITIONS)?;
            DEFAULT_GAME_DEFINITIONS.to_string()
        }
        Err(e) => return Err(e.into()),
    };
    let defs: Vec<GameDefinition> = serde_json::from_str(&raw)?;
    Ok(defs)
}

pub fn fetch_servers(definition: &GameDefinition) -> Result<Vec<Server>, FetchError> {
    if let Some(servers) = get_cached_servers(definition.app_id) {
        return Ok(servers);
    }

    let url = format!(
        "https://api.steampowered.com/ISteamApps/GetSDRConfig/v1/?appid={}",
        definition.app_id
    );

    let mut response = HTTP_AGENT.get(&url).call()?;

    let status = response.status();
    if status != 200 {
        return Err(FetchError::Status(status.as_u16()));
    }

    let config: SdrConfig = response.body_mut().read_json()?;

    let mut servers = Vec::with_capacity(config.pops.len());
    let mut clusters: HashMap<String, Vec<Relay>> = HashMap::new();

    for (pop_id, pop_def) in &config.pops {
        let desc = &pop_def.desc;

        let relays = match &pop_def.relays {
            Some(relay_defs) => relay_defs.clone(),
            None => continue,
        };

        if relays.is_empty() {
            continue;
        }

        let matched_cluster = definition
            .cluster_keywords
            .iter()
            .find(|keyword| desc.contains(*keyword));

        if let Some(cluster_name) = matched_cluster {
            clusters
                .entry(cluster_name.clone())
                .or_default()
                .extend(relays);
        } else {
            let s = Server {
                id: pop_id.clone(),
                description: desc.clone(),
                relays,
                ping: None,
                status_text: String::new(),
                blocked: false,
                packet_loss: None,
            };
            if s.is_accepted(definition) {
                servers.push(s);
            }
        }
    }

    for (cluster_name, relays) in clusters {
        let s = Server {
            id: "cluster".to_string(),
            description: cluster_name,
            relays,
            ping: None,
            status_text: String::new(),
            blocked: false,
            packet_loss: None,
        };
        if s.is_accepted(definition) {
            servers.push(s);
        }
    }

    servers.sort_by(|a, b| a.description.cmp(&b.description));
    put_cached_servers(definition.app_id, servers.clone());

    Ok(servers)
}
