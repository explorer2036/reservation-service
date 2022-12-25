use crate::{ReservationStatus, SqlxReservationStatus};
use std::fmt;

impl From<SqlxReservationStatus> for ReservationStatus {
    fn from(status: SqlxReservationStatus) -> Self {
        match status {
            SqlxReservationStatus::Pending => ReservationStatus::Pending,
            SqlxReservationStatus::Blocked => ReservationStatus::Blocked,
            SqlxReservationStatus::Confirmed => ReservationStatus::Confirmed,
            SqlxReservationStatus::Unknown => ReservationStatus::Unknown,
        }
    }
}

impl fmt::Display for ReservationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReservationStatus::Pending => write!(f, "pending"),
            ReservationStatus::Blocked => write!(f, "blocked"),
            ReservationStatus::Confirmed => write!(f, "confirmed"),
            ReservationStatus::Unknown => write!(f, "unknown"),
        }
    }
}
