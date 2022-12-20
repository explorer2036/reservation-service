mod error;
mod store;

pub use error::ReservationError;

use async_trait::async_trait;
use sqlx::PgPool;

#[derive(Debug)]
pub struct ReservationStore {
    _pool: PgPool,
}

#[async_trait]
pub trait Reservation {
    /// make a reservation
    async fn reserve(
        &self,
        reservation: abi::Reservation,
    ) -> Result<abi::Reservation, ReservationError>;
    /// change reservation status to confirmed if the current status is pending
    async fn confirm(&self, id: String) -> Result<abi::Reservation, ReservationError>;
    /// update note
    async fn update(&self, id: String, note: String) -> Result<abi::Reservation, ReservationError>;
    /// delete reservation
    async fn delete(&self, id: String) -> Result<abi::Reservation, ReservationError>;
    /// get reservation by id
    async fn get(&self, id: String) -> Result<abi::Reservation, ReservationError>;
    /// query reservations
    async fn query(
        &self,
        query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, ReservationError>;
    /// query reservations order by reservation id
    async fn filter(
        &self,
        query: abi::ReservationFilter,
    ) -> Result<Vec<abi::Reservation>, ReservationError>;
}
