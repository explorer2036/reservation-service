use crate::{Reservation, ReservationError, ReservationStore};
use async_trait::async_trait;

#[async_trait]
impl Reservation for ReservationStore {
    async fn reserve(
        &self,
        _reservation: abi::Reservation,
    ) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }

    async fn confirm(&self, _id: String) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }

    async fn update(
        &self,
        _id: String,
        _note: String,
    ) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }

    async fn delete(&self, _id: String) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }

    async fn get(&self, _id: String) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }

    async fn query(
        &self,
        _query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, ReservationError> {
        todo!()
    }

    async fn filter(
        &self,
        _query: abi::ReservationFilter,
    ) -> Result<Vec<abi::Reservation>, ReservationError> {
        todo!()
    }
}
