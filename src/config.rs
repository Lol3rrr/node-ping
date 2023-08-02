use std::net::IpAddr;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigNode {
    /// The Name to display in the notifications regarding the Node
    pub name: String,
    /// The Address to attempt and ping to determine the uptime/availability of the Node
    pub addr: IpAddr,
}

#[derive(Debug, Clone, Deserialize)]
pub enum NotificationTarget {
    /// Sends the Notifications as a discord message using the provided Webhook
    DiscordWebhook {
        /// The URL of the Webhook
        url: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// The Nodes to monitor
    pub nodes: Vec<ConfigNode>,
    /// How you want to be notified of any changes
    pub notify_targets: Vec<NotificationTarget>,
    /// The interval at which to check the Nodes
    #[serde(default = "default_ping_interval")]
    pub ping_interval: u64,
}

pub fn default_ping_interval() -> u64 {
    30
}
