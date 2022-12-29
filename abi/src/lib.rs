mod config;
mod error;
mod pb;
mod types;

pub use config::*;
pub use error::*;
pub use pb::*;

use chrono::{DateTime, NaiveDateTime, Utc};
use prost_types::Timestamp;

pub type ReservationId = i64;
pub type UserId = String;
pub type ResourceId = String;

/// validate the data structure, raise error if invalid
pub trait Validator {
    fn validate(&self) -> Result<(), Error>;
}

/// validate and normalize the data structure
pub trait Normalizer: Validator {
    /// caller should call normalize to make sure the data structure is ready to use
    fn normalize(&mut self) -> Result<(), Error> {
        self.validate()?;
        self.do_normalize();
        Ok(())
    }

    /// user shall implement do_normalize() to normalize the data structure
    fn do_normalize(&mut self);
}

pub trait ToSql {
    fn to_sql(&self) -> String;
}

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

impl Validator for ReservationId {
    fn validate(&self) -> Result<(), Error> {
        if *self <= 0 {
            Err(Error::InvalidReservationId(*self))
        } else {
            Ok(())
        }
    }
}
