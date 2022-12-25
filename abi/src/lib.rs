mod error;
mod pb;
mod types;

pub use error::*;
pub use pb::*;

use chrono::{DateTime, NaiveDateTime, Utc};
use prost_types::Timestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "reservation_status", rename_all = "lowercase")]
pub enum SqlxReservationStatus {
    Unknown,
    Pending,
    Confirmed,
    Blocked,
}

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

pub fn convert_to_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}
