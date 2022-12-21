use crate::{Reservation, ReservationStore};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::types::PgRange, Row};

#[async_trait]
impl Reservation for ReservationStore {
    async fn reserve(
        &self,
        mut reservation: abi::Reservation,
    ) -> Result<abi::Reservation, abi::Error> {
        let start = abi::convert_to_utc_time(reservation.start.as_ref());
        let end = abi::convert_to_utc_time(reservation.end.as_ref());
        let timespan: PgRange<DateTime<Utc>> = (start..end).into();

        // make a insert sql for the reservation
        let sql = "INSERT INTO reservations (user_id, resource_id, timespan, note, status) VALUES ($1, $2, $3, $4, $5) RETURNING id";
        let id = sqlx::query(sql)
            .bind(reservation.user_id.clone())
            .bind(reservation.resource_id.clone())
            .bind(timespan)
            .bind(reservation.note.clone())
            .bind(reservation.status)
            .fetch_one(&self.pool)
            .await?
            .get(0);
        reservation.id = id;
        Ok(reservation)
    }

    async fn confirm(&self, _id: String) -> Result<abi::Reservation, abi::Error> {
        todo!()
    }

    async fn update(&self, _id: String, _note: String) -> Result<abi::Reservation, abi::Error> {
        todo!()
    }

    async fn delete(&self, _id: String) -> Result<abi::Reservation, abi::Error> {
        todo!()
    }

    async fn get(&self, _id: String) -> Result<abi::Reservation, abi::Error> {
        todo!()
    }

    async fn query(
        &self,
        _query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, abi::Error> {
        todo!()
    }

    async fn filter(
        &self,
        _query: abi::ReservationFilter,
    ) -> Result<Vec<abi::Reservation>, abi::Error> {
        todo!()
    }
}
