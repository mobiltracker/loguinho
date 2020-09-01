use clap::Clap;
use rusoto_core::{
    credential::{ChainProvider, ProfileProvider},
    HttpClient, Region,
};
use rusoto_logs::CloudWatchLogsClient;

use helpers::ping_client;
use std::{error::Error, sync::Arc, time::Duration};
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
    smol::run(async { main_async().await.unwrap() });
}

async fn main_async() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();

    colored::control::set_virtual_terminal(true).unwrap();

    let mut profile = ProfileProvider::new()?;
    profile.set_profile("loguinho");
    let mut chain = ChainProvider::with_profile_provider(profile);
    chain.set_timeout(Duration::from_secs(2));
    let dispatcher = Arc::new(HttpClient::new()?);

    let mut client = CloudWatchLogsClient::new_with(dispatcher, chain, Region::SaEast1);

    match ping_client(&client).await {
        Ok(_) => {}
        Err(_) => {
            let dispatcher = Arc::new(HttpClient::new()?);

            println!("Missing [loguinho] profile for aws credentials -> Trying default");

            let mut chain = ChainProvider::new();
            chain.set_timeout(Duration::from_secs(2));

            client = CloudWatchLogsClient::new_with(dispatcher, chain, Region::SaEast1)
        }
    }

    match opts.subcmd {
        SubCommand::Watch(w) => watch_main(w, client).await?,
    }

    Ok(())
}
