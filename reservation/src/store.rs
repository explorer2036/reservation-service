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
    use std::path::Path;

    use super::*;
    use abi::{ReservationConflict, ReservationConflictInfo, ReservationWindow};
    use sqlx::PgPool;
    use sqlx_db_tester::TestPg;

    #[tokio::test]
    async fn reserve_should_work_for_valid_window() {
        let db = init_db();
        let pool = db.get_pool().await;
        let (reservation, _store) = make_alon_reservation(pool).await;
        assert!(reservation.id > 0);
    }

    #[tokio::test]
    async fn reserve_conflict_reservation_should_reject() {
        let db = init_db();
        let pool = db.get_pool().await;
        let (_r1, s1) = make_alon_reservation(pool.clone()).await;
        let r2 = abi::Reservation::new_pending(
            "alon".to_string(),
            "ocean-view-room-711".to_string(),
            "2022-12-26T15:00:00-0700".parse().unwrap(),
            "2022-12-30T15:00:00-0700".parse().unwrap(),
            "note".to_string(),
        );
        let err = s1.reserve(r2).await.unwrap_err();
        let info = ReservationConflictInfo::Parsed(ReservationConflict {
            new: ReservationWindow {
                rid: "ocean-view-room-711".to_string(),
                start: "2022-12-26T15:00:00-0700".parse().unwrap(),
                end: "2022-12-30T15:00:00-0700".parse().unwrap(),
            },
            old: ReservationWindow {
                rid: "ocean-view-room-711".to_string(),
                start: "2022-12-25T15:00:00-0700".parse().unwrap(),
                end: "2022-12-28T15:00:00-0700".parse().unwrap(),
            },
        });
        assert_eq!(err, abi::Error::ConflictReservation(info));
    }

    // private none test functions
    fn init_db() -> TestPg {
        TestPg::new(
            "postgres://postgres:123456@localhost:5432".to_string(),
            Path::new("../migrations"),
        )
    }

    async fn make_alon_reservation(pool: PgPool) -> (abi::Reservation, ReservationStore) {
        make_reservation(
            pool,
            "alon",
            "ocean-view-room-711",
            "2022-12-25T15:00:00-0700",
            "2022-12-28T15:00:00-0700",
            "note",
        )
        .await
    }

    async fn _make_alice_reservation(pool: PgPool) -> (abi::Reservation, ReservationStore) {
        make_reservation(
            pool,
            "alice",
            "ocean-view-room-713",
            "2023-01-25T15:00:00-0700",
            "2023-02-28T15:00:00-0700",
            "note",
        )
        .await
    }

    async fn make_reservation(
        pool: PgPool,
        uid: &str,
        rid: &str,
        start: &str,
        end: &str,
        note: &str,
    ) -> (abi::Reservation, ReservationStore) {
        let store = ReservationStore::new(pool.clone());
        let reservation = abi::Reservation::new_pending(
            uid,
            rid,
            start.parse().unwrap(),
            end.parse().unwrap(),
            note,
        );
        (store.reserve(reservation).await.unwrap(), store)
    }
}
