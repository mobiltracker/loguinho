use clap::Clap;
use rusoto_core::{credential::ChainProvider, HttpClient, Region};
use rusoto_logs::{
    CloudWatchLogs, CloudWatchLogsClient, DescribeLogGroupsRequest, GetLogEventsRequest, LogGroup,
};
use std::{error::Error, sync::Arc};

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
    let log_grouos = get_all_log_groups(&client, &w.input).await.unwrap();
    println!("{:?}", log_grouos);
    // loop {
    //     for log_group in wanted_log_groups.iter() {
    //         let curr_time = chrono::offset::Utc::now();
    //     }
    // }
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

async fn get_all_log_groups(
    client: &CloudWatchLogsClient,
    filter: &str,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut log_groups_vector: Vec<LogGroup> = vec![];
    let mut log_group_response = client
        .describe_log_groups(DescribeLogGroupsRequest {
            limit: Some(1),
            log_group_name_prefix: Some("/ecs/".to_owned()),
            next_token: None,
        })
        .await?;

    if log_group_response.log_groups.is_none() {
        return Ok(vec![]);
    }

    if log_group_response.next_token.is_none() {
        return Ok(vec![log_group_response
            .log_groups
            .unwrap()
            .first()
            .unwrap()
            .log_group_name
            .clone()
            .unwrap()]);
    }

    while let Some(next_token) = &log_group_response.next_token {
        if let Some(log_groups) = log_group_response.log_groups.as_mut() {
            log_groups_vector.append(log_groups);
        }

        log_group_response = client
            .describe_log_groups(DescribeLogGroupsRequest {
                limit: Some(10),
                log_group_name_prefix: Some("/ecs/".to_owned()),
                next_token: Some(next_token.to_owned()),
            })
            .await?
    }

    let wanted_log_groups = log_groups_vector
        .iter()
        .filter(|item| {
            if let Some(log_group_name) = item.log_group_name.clone() {
                log_group_name.contains(filter)
            } else {
                false
            }
        })
        .map(|item| {
            item.log_group_name
                .clone()
                .unwrap_or("missing log group name".to_owned())
        })
        .collect::<Vec<String>>();

    Ok(wanted_log_groups)
}
