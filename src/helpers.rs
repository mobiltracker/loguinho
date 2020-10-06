use rusoto_core::RusotoError;
use rusoto_logs::{
    CloudWatchLogs, CloudWatchLogsClient, DescribeLogGroupsRequest, DescribeLogStreamsRequest,
    DescribeLogStreamsResponse, FilterLogEventsError, FilterLogEventsRequest, FilteredLogEvent,
    LogGroup,
};

use std::error::Error;
use std::time::Duration;

pub async fn get_all_log_groups<'a>(
    client: &CloudWatchLogsClient,
    filter: &str,
) -> Result<Vec<LogGroup>, Box<dyn Error>> {
    let mut log_groups_vector: Vec<LogGroup> = vec![];
    let mut log_group_response = client
        .describe_log_groups(DescribeLogGroupsRequest {
            limit: Some(1),
            log_group_name_prefix: None,
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
                limit: Some(50),
                log_group_name_prefix: None,
                next_token: Some(next_token.to_owned()),
            })
            .await?;

        if log_group_response.next_token.is_none() && log_group_response.log_groups.is_some() {
            log_groups_vector.append(&mut log_group_response.log_groups.unwrap());
            break;
        }
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

#[allow(dead_code)]
pub async fn get_all_log_streams(
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

pub async fn get_last_events_from_log_group(
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

pub async fn ping_client(client: &CloudWatchLogsClient) -> Result<(), Box<dyn Error>> {
    client
        .describe_log_groups(DescribeLogGroupsRequest {
            limit: Some(1),
            log_group_name_prefix: None,
            next_token: None,
        })
        .await?;

    Ok(())
}
