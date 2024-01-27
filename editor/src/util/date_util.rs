use chrono::{DateTime, Utc};

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
