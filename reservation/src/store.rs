use crate::{Reservation, ReservationStore};
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl Reservation for ReservationStore {
    async fn reserve(
        &self,
        mut reservation: abi::Reservation,
    ) -> Result<abi::Reservation, abi::Error> {
        reservation.validate()?;

        let timespan = reservation.get_timespan();
        let status = abi::ReservationStatus::from_i32(reservation.status)
            .unwrap_or(abi::ReservationStatus::Pending);
        // make a insert sql for the reservation
        let sql = "INSERT INTO reservations (user_id, resource_id, timespan, note, status) VALUES ($1, $2, $3, $4, $5::reservation_status) RETURNING id";
        let id = sqlx::query(sql)
            .bind(reservation.user_id.clone())
            .bind(reservation.resource_id.clone())
            .bind(timespan)
            .bind(reservation.note.clone())
            .bind(status.to_string())
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

impl ReservationStore {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_window() {
        let store = ReservationStore::new(migrated_pool.clone());
        let reservation = abi::Reservation::new_pending(
            "alon".to_string(),
            "ocean-view-room-713".to_string(),
            "2022-12-25T15:00:00-0700".parse().unwrap(),
            "2022-12-28T15:00:00-0700".parse().unwrap(),
            "note".to_string(),
        );
        let result = store.reserve(reservation).await.unwrap();
        assert!(result.id > 0)
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_conflict_reservation_should_reject() {
        let store = ReservationStore::new(migrated_pool.clone());
        let r1 = abi::Reservation::new_pending(
            "alon".to_string(),
            "ocean-view-room-713".to_string(),
            "2022-12-25T15:00:00-0700".parse().unwrap(),
            "2022-12-28T15:00:00-0700".parse().unwrap(),
            "note".to_string(),
        );
        let r2 = abi::Reservation::new_pending(
            "belayu".to_string(),
            "ocean-view-room-713".to_string(),
            "2022-12-26T15:00:00-0700".parse().unwrap(),
            "2022-12-30T15:00:00-0700".parse().unwrap(),
            "note".to_string(),
        );
        let _ = store.reserve(r1).await.unwrap();
        let err = store.reserve(r2).await.unwrap_err();
        println!("{:?}", err);
    }
}
