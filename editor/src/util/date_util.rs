use std::time::Duration;

use chrono::{DateTime, Utc};

/// "45s", "12m" or "1h 05m"; seconds and minutes are rounded up.
pub fn format_plot_duration(duration: Duration) -> String {
    let seconds = duration.as_secs_f64().ceil() as u64;
    if seconds < 60 {
        return format!("{seconds}s");
    }

    let total_minutes = seconds.div_ceil(60);
    let (hours, minutes) = (total_minutes / 60, total_minutes % 60);
    match (hours, minutes) {
        (0, _) => format!("{minutes}m"),
        (_, 0) => format!("{hours}h"),
        _ => format!("{hours}h {minutes:02}m"),
    }
}

fn format_time_unit_and_number(unit: &str, number: i64) -> String {
    format!("{} {}{}", number, unit, if number > 1 { "s" } else { "" })
}

pub fn format_datetime_to_relative(date_time: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*date_time);
    let seconds = duration.num_seconds();
    let minutes = duration.num_minutes();
    let hours = duration.num_hours();
    let days = duration.num_days();
    let weeks = duration.num_weeks();
    let months = days / 30;
    let years = days / 365;

    if years != 0 {
        format_time_unit_and_number("year", years)
    } else if months != 0 {
        format_time_unit_and_number("month", months)
    } else if weeks != 0 {
        format_time_unit_and_number("week", weeks)
    } else if days != 0 {
        format_time_unit_and_number("day", days)
    } else if hours != 0 {
        format_time_unit_and_number("hour", hours)
    } else if minutes != 0 {
        format_time_unit_and_number("minute", minutes)
    } else {
        format_time_unit_and_number("second", seconds)
    }
}
