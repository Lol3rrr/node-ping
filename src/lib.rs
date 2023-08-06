use std::time::Duration;

use node::Node;
use rand::Rng;

use crate::{config::NotificationTarget, node::NodeStatus};

pub mod config;
pub mod node;

/// The Client running all the checks
pub struct Client {
    /// The Root config for the Client
    config: config::Config,
    /// The Nodes that are being tracked and their corresponding status/other state
    nodes: Vec<node::Node>,
}

impl From<config::Config> for Client {
    fn from(value: config::Config) -> Self {
        let nodes: Vec<_> = value
            .nodes
            .iter()
            .cloned()
            .map(|c| node::Node::from(c))
            .collect();

        Self {
            config: value,
            nodes,
        }
    }
}

#[derive(Debug)]
pub enum NotifyMessages {
    Pending { node: Node },
    Down { node: Node },
    BackUp { node: Node },
}

impl Client {
    pub async fn run(mut self) {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        let checking_handle = tokio::spawn(async move {
            loop {
                tracing::info!("Checking Nodes");

                for node in self.nodes.iter_mut() {
                    let prev_state = node.status();

                    match node.check().await {
                        Ok(nstate) => match nstate {
                            NodeStatus::Unknown => {}
                            NodeStatus::Up => {
                                if prev_state == NodeStatus::Down
                                    || prev_state == NodeStatus::Pending
                                {
                                    tx.send(NotifyMessages::BackUp { node: node.clone() })
                                        .expect("The Notify Task should never stop");
                                }
                            }
                            NodeStatus::Down => {
                                if prev_state != NodeStatus::Down {
                                    tx.send(NotifyMessages::Down { node: node.clone() })
                                        .expect("The Notify Task should never stop");
                                }
                            }
                            NodeStatus::Pending => {
                                tx.send(NotifyMessages::Pending { node: node.clone() })
                                    .expect("The Notify Task should never stop");
                            }
                        },
                        Err(e) => {
                            tracing::error!("Pinging: {:?}", e);
                        }
                    };

                    // Add some delay between checking different Nodes
                    let duration = Duration::from_millis(rand::thread_rng().gen_range(2..6));
                    tokio::time::sleep(duration).await;
                }

                // Add some random jitter to the base delay betwen checking nodes to avoid large
                // spikes
                let base_duration_ms: u64 = self.config.ping_interval * 1000;
                let jitter: i64 = rand::thread_rng().gen_range(0..250);
                let duration_ms = base_duration_ms
                    .checked_add_signed(jitter)
                    .unwrap_or(base_duration_ms);
                let duration = Duration::from_millis(duration_ms);

                tracing::info!("Done checking Nodes, waiting {:?}", duration);

                tokio::time::sleep(duration).await;
            }
        });

        let notify_handle = tokio::spawn(async move {
            let client = reqwest::Client::new();

            let bot_name = format!("Node-Up-Bot [{:?}]", gethostname::gethostname());

            while let Some(msg) = rx.recv().await {
                tracing::debug!("Notify {:?}", msg);

                for target in self.config.notify_targets.iter() {
                    match target {
                        NotificationTarget::DiscordWebhook { url } => {
                            let node = match &msg {
                                NotifyMessages::BackUp { node } => node,
                                NotifyMessages::Pending { node } => node,
                                NotifyMessages::Down { node } => node,
                            };

                            let value = serde_json::json!({
                                "username": bot_name,
                                "embeds": [{
                                    "title": "Node Info",
                                    "fields": [{
                                        "name": "Name",
                                        "value": format!("{:?}", node.name()),
                                    }, {
                                        "name": "IP",
                                        "value": format!("{}", node.addr()),
                                    }, {
                                        "name": "Status",
                                        "value": format!("{:?}", node.status()),
                                    }],
                                }],
                            });

                            match client.post(url).json(&value).send().await {
                                Ok(_r) => {}
                                Err(e) => {
                                    tracing::error!("Sending Webhook: {:?}", e);
                                }
                            };
                        }
                    };
                }
            }
        });

        let _ = tokio::join!(checking_handle, notify_handle);
    }
}
