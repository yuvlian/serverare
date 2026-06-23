use std::collections::HashMap;
use std::net::IpAddr;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct GameDefinition {
    #[serde(rename = "gameMode")]
    pub game_mode: String,
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "appId")]
    pub app_id: u32,
    #[serde(rename = "keywordFilterMode")]
    pub keyword_filter_mode: String,
    pub keywords: Vec<String>,
    #[serde(rename = "clusterKeywords")]
    pub cluster_keywords: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SdrConfig {
    // pub revision: serde_json::Value,
    pub pops: HashMap<String, PopDefinition>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PopDefinition {
    pub desc: String,
    pub relays: Option<Vec<Relay>>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Relay {
    pub ipv4: IpAddr,
}

#[derive(Debug, Clone)]
pub struct Server {
    pub id: String,
    pub description: String,
    pub relays: Vec<Relay>,
    pub ping: Option<u32>,
    pub status_text: String,
    pub blocked: bool,
    pub packet_loss: Option<u32>,
}

impl Server {
    pub fn is_accepted(&self, definition: &GameDefinition) -> bool {
        let desc_lower = self.description.to_lowercase();
        match definition.keyword_filter_mode.to_lowercase().as_str() {
            "include" => definition
                .keywords
                .iter()
                .any(|k| desc_lower.contains(&k.to_lowercase())),
            "exclude" => !definition
                .keywords
                .iter()
                .any(|k| desc_lower.contains(&k.to_lowercase())),
            _ => true,
        }
    }
}
