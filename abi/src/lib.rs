mod error;
mod pb;

pub use error::*;
pub use pb::*;

use chrono::{DateTime, NaiveDateTime, Utc};
use prost_types::Timestamp;

pub fn convert_to_utc_time(ts: Option<&Timestamp>) -> DateTime<Utc> {
    DateTime::<Utc>::from_utc(
        {
            let secs = ts.unwrap().seconds;
            let nsecs = ts.unwrap().nanos as _;
            let datetime = NaiveDateTime::from_timestamp_opt(secs, nsecs);
            datetime.unwrap()
        },
        Utc,
    )
}
