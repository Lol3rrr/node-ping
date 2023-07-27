use std::{net::IpAddr, time::Duration};

use crate::config;

/// The Status of a Node
#[derive(Debug, Clone, PartialEq)]
pub enum NodeStatus {
    /// An unknown State indicates that we have not yet checked the Node
    Unknown,
    /// The Node is up and responding to Pings
    Up,
    /// The Node has failed our last ping request
    Pending,
    /// The Node has failed multiple ping requests and is therefore assumed to be down
    Down,
}

#[derive(Debug, Clone)]
pub struct Node {
    name: String,
    addr: IpAddr,
    status: NodeStatus,
}

impl From<config::ConfigNode> for Node {
    fn from(value: config::ConfigNode) -> Self {
        Self {
            name: value.name,
            addr: value.addr,
            status: NodeStatus::Unknown,
        }
    }
}

impl Node {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn status(&self) -> NodeStatus {
        self.status.clone()
    }

    pub fn addr(&self) -> &IpAddr {
        &self.addr
    }

    pub async fn check(&mut self) -> Result<NodeStatus, ()> {
        let pinger = match tokio_icmp_echo::Pinger::new().await {
            Ok(p) => p,
            Err(e) => {
                println!("Error creating Pinger: {:?}", e);
                return Err(());
            }
        };

        match pinger
            .ping(self.addr, 0, 0, Duration::from_millis(250))
            .await
        {
            Ok(Some(_t)) => {
                self.status = NodeStatus::Up;
            }
            Ok(None) => {
                let n_status = match self.status {
                    NodeStatus::Unknown | NodeStatus::Up => NodeStatus::Pending,
                    NodeStatus::Pending | NodeStatus::Down => NodeStatus::Down,
                };
                self.status = n_status;
            }
            Err(e) => {
                println!("Pinging: {:?}", e);
                return Err(());
            }
        };

        Ok(self.status.clone())
    }
}
