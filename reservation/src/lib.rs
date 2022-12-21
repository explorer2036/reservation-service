mod store;

use async_trait::async_trait;
use sqlx::PgPool;

#[derive(Debug)]
pub struct ReservationStore {
    pool: PgPool,
}

#[async_trait]
pub trait Reservation {
    /// make a reservation
    async fn reserve(&self, reservation: abi::Reservation) -> Result<abi::Reservation, abi::Error>;
    /// change reservation status to confirmed if the current status is pending
    async fn confirm(&self, id: String) -> Result<abi::Reservation, abi::Error>;
    /// update note
    async fn update(&self, id: String, note: String) -> Result<abi::Reservation, abi::Error>;
    /// delete reservation
    async fn delete(&self, id: String) -> Result<abi::Reservation, abi::Error>;
    /// get reservation by id
    async fn get(&self, id: String) -> Result<abi::Reservation, abi::Error>;
    /// query reservations
    async fn query(
        &self,
        query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, abi::Error>;
    /// query reservations order by reservation id
    async fn filter(
        &self,
        query: abi::ReservationFilter,
    ) -> Result<Vec<abi::Reservation>, abi::Error>;
}
