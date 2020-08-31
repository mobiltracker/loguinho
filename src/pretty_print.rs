use chrono::{DateTime, Utc};
use colored::*;
use rusoto_logs::FilteredLogEvent;

pub fn pretty_print_log_event(log_event: &FilteredLogEvent, log_group_name: Option<&str>) {
    let log_name = log_group_name.unwrap_or("missing name");
    let log_message = log_event
        .message
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("missing message");

    let date_time: DateTime<Utc> = chrono::DateTime::from_utc(
        chrono::NaiveDateTime::from_timestamp(
            log_event.timestamp.expect("missing timestamp") / 1000,
            0,
        ),
        Utc,
    );

    println!(
        "{} {} {}",
        log_name.bright_cyan(),
        date_time.to_string().bright_red(),
        log_message.bright_green()
    );
}
