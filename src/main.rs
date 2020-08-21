use chrono::NaiveDateTime;
use clap::Clap;
use rusoto_core::{credential::ChainProvider, HttpClient, Region};
use rusoto_logs::{
    CloudWatchLogs, CloudWatchLogsClient, DescribeLogGroupsRequest, GetLogEventsRequest,
};
use std::sync::Arc;

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
struct Watch {
    input: String,
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
        SubCommand::Watch(w) => watch_main(w, client).await,
    }
}

async fn watch_main(w: Watch, client: CloudWatchLogsClient) {
    let describe_request = DescribeLogGroupsRequest {
        limit: Some(50),
        log_group_name_prefix: Some("/ecs/".to_owned()),
        next_token: None,
    };

    let log_groups = client
        .describe_log_groups(describe_request)
        .await
        .expect("failed to get log groups");

    let wanted_log_groups = log_groups
        .log_groups
        .expect("no logs for selected group")
        .iter()
        .filter(|item| {
            if let Some(log_group_name) = item.log_group_name.clone() {
                println!("lg: {:?} w:{:?}", log_group_name, w.input);
                log_group_name.contains(&w.input)
            } else {
                false
            }
        })
        .map(|item| item.log_group_name.clone().unwrap())
        .collect::<Vec<String>>();

    let mut logs: Vec<String> = vec![];

    loop {
        for log_group in wanted_log_groups.iter() {
            let curr_time = chrono::offset::Utc::now();
        }
    }
}

// .map(|item| {
//     let curr_time = chrono::offset::Utc::now();
//     let log_request = GetLogEventsRequest {
//         start_time: curr_time.timestamp_millis(),
//         end_time: None,
//         limit:
//     };
//     // client.get_log_events(input)
//     todo!()
// })
