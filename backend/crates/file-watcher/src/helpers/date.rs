use chrono::prelude::*;

pub fn parse_db_date(date: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    date.parse::<DateTime<Utc>>().or_else(|_| {
        NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M:%S%.f").map(|x| x.and_utc())
    })
}
