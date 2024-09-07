use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{self, Deserialize, Deserializer, Serializer};
pub mod redis_util;
use crate::common::common_constants::COMMON_TIME_FORMAT;
use time::OffsetDateTime;
use time::UtcOffset;
const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

// The signature of a serialize_with function must follow the pattern:
//
//    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
//    where
//        S: Serializer
//
// although it may also be generic over the input types T.
pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = format!("{}", date.format(FORMAT));
    serializer.serialize_str(&s)
}

// The signature of a deserialize_with function must follow the pattern:
//
//    fn deserialize<'de, D>(D) -> Result<T, D::Error>
//    where
//        D: Deserializer<'de>
//
// although it may also be generic over the output types T.
pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let dt = NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
    Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
}
pub fn serialize_human_readable_time<S>(
    time: &OffsetDateTime,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Get the local offset (e.g., UTC+2)
    let local_offset = UtcOffset::current_local_offset().map_err(serde::ser::Error::custom)?;
    // Convert the time to local time
    let local_time = time.to_offset(local_offset);

    // Format the time in a human-readable way

    // Format the OffsetDateTime to a string
    let time_str = local_time
        .format(&COMMON_TIME_FORMAT)
        .map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&time_str)
}
