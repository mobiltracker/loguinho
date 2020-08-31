use clap::Clap;
use rusoto_core::{credential::ChainProvider, HttpClient, Region};
use rusoto_logs::CloudWatchLogsClient;

use std::sync::Arc;
use watch::watch_main;

mod helpers;
mod pretty_print;
mod watch;

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
#[clap(version = "1.0", author = "Matheus Cruz <mlcruz@inf.ufrgs.br>")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(version = "1.3", author = "Matheus Cruz <mlcruz@inf.ufrgs.br>")]
    Watch(Watch),
}

/// A subcommand for controlling testing
#[derive(Clap)]
pub struct Watch {
    input: Option<String>,
}

fn main() {
    smol::run(async { main_async().await });
}

async fn main_async() {
    let opts: Opts = Opts::parse();

    let chain = ChainProvider::new();
    let dispatcher = Arc::new(HttpClient::new().expect("failed to create request dispatcher"));

    let client = CloudWatchLogsClient::new_with(dispatcher, chain, Region::SaEast1);

    match opts.subcmd {
        SubCommand::Watch(w) => watch_main(w, client).await.unwrap(),
    }
}
