use rusoto_core::RusotoError;
use rusoto_logs::CloudWatchLogsClient;

use crate::{
    helpers::{get_all_log_groups, get_last_events_from_log_group, sleep},
    pretty_print::pretty_print_log_event,
    Watch,
};
use std::error::Error;
use std::{collections::HashSet, time::Duration};

pub async fn watch_main(w: Watch, client: CloudWatchLogsClient) -> Result<(), Box<dyn Error>> {
    let log_groups = get_all_log_groups(&client, &w.input.unwrap_or("".to_owned())).await?;
    let mut log_events_filter: HashSet<Option<String>> = HashSet::new();

    println!("{:?}", log_groups);
    loop {
        let curr_timestamp = chrono::offset::Utc::now().timestamp_millis() - 30000;

        for log_group_chunk in log_groups.chunks(5) {
            for log_group in log_group_chunk {
                let events_result =
                    get_last_events_from_log_group(&client, log_group, curr_timestamp).await;

                match events_result {
                    Ok(events) => {
                        if !events.is_empty() {
                            let log_name = log_group.log_group_name.as_ref().map(|s| s.as_str());

                            events.iter().for_each(|event| {
                                if !log_events_filter.contains(&event.event_id) {
                                    pretty_print_log_event(event, log_name);
                                    log_events_filter.insert(event.event_id.clone());
                                }
                            });
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
                sleep(Duration::from_millis(50)).await;
            }
            sleep(Duration::from_millis(250)).await;
        }

        sleep(Duration::from_secs(5)).await;

        if log_events_filter.len() > 100000 {
            println!("clearing log filter...");
            log_events_filter = HashSet::new();
            sleep(Duration::from_secs(20)).await;
        }
    }
}
