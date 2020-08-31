use clap::Clap;
use rusoto_core::{credential::ChainProvider, HttpClient, Region, RusotoError};
use rusoto_logs::{
    CloudWatchLogs, CloudWatchLogsClient, DescribeLogGroupsRequest, DescribeLogStreamsRequest,
    DescribeLogStreamsResponse, FilterLogEventsError, FilterLogEventsRequest, FilteredLogEvent,
    LogGroup,
};

use std::time::Duration;
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

async fn watch_main(w: Watch, client: CloudWatchLogsClient) -> Result<(), Box<dyn Error>> {
    let log_groups = get_all_log_groups(&client, &w.input.unwrap_or("".to_owned())).await?;

    loop {
        let curr_timestamp = chrono::offset::Utc::now().timestamp_millis() - 10000;

        for log_group_chunk in log_groups.chunks(5) {
            for log_group in log_group_chunk {
                let events_result =
                    get_last_events_from_log_group(&client, log_group, curr_timestamp).await;

                match events_result {
                    Ok(events) => {
                        if !events.is_empty() {
                            println!("{:?}", events);
                        }
                    }
                    Err(err) => match err {
                        RusotoError::Credentials(_) => {
                            println!("missing credentials, {}", err.to_string())
                        }
                        RusotoError::Unknown(uerr) => {
                            if uerr.body_as_str().contains("Rate exceeded") {
                                println!("rate exceeded, waiting 1");
                                sleep(Duration::from_secs(1)).await;
                            } else {
                                println!("unknown error: {}", uerr.body_as_str());
                            }
                        }
                        _ => println!("error: {}", err.to_string()),
                    },
                }
            }
            sleep(Duration::from_millis(500)).await;
        }

        sleep(Duration::from_secs(10)).await;
    }

    // let events = get_last_events_from_log_groups(&client, &log_groups).await?;
    //println!("{:?}", events);
    // loop {
    //     for log_group in wanted_log_groups.iter() {
    //         let curr_time = chrono::offset::Utc::now();
    //     }
    // }

    Ok(())
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

async fn get_all_log_groups<'a>(
    client: &CloudWatchLogsClient,
    filter: &str,
) -> Result<Vec<LogGroup>, Box<dyn Error>> {
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
        let log_group = log_group_response
            .log_groups
            .unwrap()
            .first()
            .unwrap()
            .to_owned();

        return Ok(vec![log_group]);
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
        .into_iter()
        .filter(|item| {
            if let Some(log_group_name) = item.log_group_name.clone() {
                log_group_name.contains(filter)
            } else {
                false
            }
        })
        .collect::<Vec<LogGroup>>();

    Ok(wanted_log_groups)
}

async fn get_all_log_streams(
    client: &CloudWatchLogsClient,
    log_groups: &Vec<LogGroup>,
) -> Result<Vec<DescribeLogStreamsResponse>, Box<dyn Error>> {
    let mut result_log_streams: Vec<DescribeLogStreamsResponse> = vec![];
    for log_group in log_groups {
        let log_stream = client
            .describe_log_streams(DescribeLogStreamsRequest {
                descending: Some(true),
                limit: Some(1),
                log_group_name: log_group
                    .log_group_name
                    .clone()
                    .expect("missing log group name"),
                log_stream_name_prefix: None,
                order_by: Some("LastEventTime".to_owned()),
                next_token: None,
            })
            .await?;

        result_log_streams.push(log_stream);
    }

    return Ok(result_log_streams);
}

async fn get_last_events_from_log_group(
    client: &CloudWatchLogsClient,
    log_group: &LogGroup,
    timestamp_millis: i64,
) -> Result<Vec<FilteredLogEvent>, RusotoError<FilterLogEventsError>> {
    let event_request = FilterLogEventsRequest {
        end_time: None,
        filter_pattern: None,
        limit: None,
        log_group_name: log_group
            .log_group_name
            .clone()
            .expect("missing log group name"),
        log_stream_name_prefix: None,
        log_stream_names: None,
        next_token: None,
        start_time: Some(timestamp_millis),
    };

    let result = client.filter_log_events(event_request).await?;

    if let Some(events) = result.events {
        return Ok(events);
    } else {
        return Ok(vec![]);
    }
}

pub async fn sleep(dur: Duration) {
    smol::Timer::new(dur).await;
}
