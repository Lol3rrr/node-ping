use std::net::IpAddr;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigNode {
    pub name: String,
    pub addr: IpAddr,
}

#[derive(Debug, Clone, Deserialize)]
pub enum NotificationTarget {
    DiscordWebhook { url: String },
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub nodes: Vec<ConfigNode>,
    pub notify_target: NotificationTarget,
    #[serde(default = "default_ping_interval")]
    pub ping_interval: u64,
}

pub fn default_ping_interval() -> u64 {
    30
}
