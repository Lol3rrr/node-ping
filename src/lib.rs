use std::time::Duration;

use node::Node;

use crate::{config::NotificationTarget, node::NodeStatus};

pub mod config;
pub mod node;

pub struct Client {
    config: config::Config,
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
                                if prev_state == NodeStatus::Down {
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
                }

                tracing::info!("Done checking Nodes, waiting 30s");

                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });

        let notify_handle = tokio::spawn(async move {
            let client = reqwest::Client::new();

            let bot_name = format!("Node-Up-Bot [{:?}]", gethostname::gethostname());

            while let Some(msg) = rx.recv().await {
                tracing::debug!("Notify {:?}", msg);

                match &self.config.notify_target {
                    NotificationTarget::DiscordWebhook { url } => {
                        let node = match &msg {
                            NotifyMessages::BackUp { node } => node,
                            NotifyMessages::Pending { node } => node,
                            NotifyMessages::Down { node } => node,
                        };

                        let content = match &msg {
                            NotifyMessages::BackUp { node } => format!("The Node {:?} is now back up", node.name()),
                            NotifyMessages::Pending { node } => format!("The Node {:?} has failed at least one ping and is now pending", node.name()),
                            NotifyMessages::Down { node } => format!("The Node {:?} has failed multiple pings and is considered down/unreachable", node.name()),
                        };

                        let title = match &msg {
                            NotifyMessages::Down { .. } => "Node Down",
                            NotifyMessages::Pending { .. } => "Node Pending",
                            NotifyMessages::BackUp { .. } => "Node Back Up",
                        };

                        let value = serde_json::json!({
                            "username": bot_name,
                            "embeds": [{
                                "title": title,
                                "description": content,
                            }, {
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
        });

        let _ = tokio::join!(checking_handle, notify_handle);
    }
}
