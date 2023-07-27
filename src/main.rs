use std::{fs::File, path::PathBuf};

use node_ping::{config::Config, Client};
use tracing::Subscriber;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version)]
struct Args {
    #[clap(long = "config", default_value = "config.yaml")]
    config: PathBuf,
}

struct CrateFilterSub {}

impl<T> tracing_subscriber::Layer<T> for CrateFilterSub
where
    T: Subscriber,
{
    fn enabled(
        &self,
        metadata: &tracing::Metadata<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, T>,
    ) -> bool {
        metadata.target().contains("node_ping")
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let tracing_layers = tracing_subscriber::registry()
        .with(CrateFilterSub {})
        .with(tracing_subscriber::fmt::Layer::new());

    tracing::subscriber::set_global_default(tracing_layers).unwrap();

    let args = Args::parse();

    let config_file = File::open(args.config).unwrap();
    let config: Config = serde_yaml::from_reader(config_file).unwrap();

    let client = Client::from(config);

    client.run().await;
}
