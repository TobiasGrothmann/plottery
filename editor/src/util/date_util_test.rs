#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use crate::util::format_datetime_to_relative;

    #[test]
    fn relative_time() {
        let past = Utc::now() - Duration::days(365 + 1);
        assert_eq!(format_datetime_to_relative(&past), "1 year");

        let past = Utc::now() - Duration::days(30);
        assert_eq!(format_datetime_to_relative(&past), "1 month");

        let past = Utc::now() - Duration::days(7);
        assert_eq!(format_datetime_to_relative(&past), "1 week");

        let past = Utc::now() - Duration::days(1);
        assert_eq!(format_datetime_to_relative(&past), "1 day");

        let past = Utc::now() - Duration::hours(1);
        assert_eq!(format_datetime_to_relative(&past), "1 hour");

        let past = Utc::now() - Duration::minutes(1);
        assert_eq!(format_datetime_to_relative(&past), "1 minute");

        let past = Utc::now() - Duration::minutes(5);
        assert_eq!(format_datetime_to_relative(&past), "5 minutes");

        let past = Utc::now() - Duration::seconds(1);
        assert_eq!(format_datetime_to_relative(&past), "1 second");

        let past = Utc::now() - Duration::seconds(30);
        assert_eq!(format_datetime_to_relative(&past), "30 seconds");
    }
}
