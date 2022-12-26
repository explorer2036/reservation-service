use crate::{Reservation, ReservationStore};
use abi::ToSql;
use async_trait::async_trait;
use futures::StreamExt;
use sqlx::{Either, Row};
use tokio::sync::mpsc;
use tracing::{info, log::warn};

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

    async fn confirm(&self, id: i64) -> Result<abi::Reservation, abi::Error> {
        let sql = "UPDATE reservations SET status = 'confirmed' WHERE id = $1 AND status = 'pending' RETURNING *";
        let reservation: abi::Reservation =
            sqlx::query_as(sql).bind(id).fetch_one(&self.pool).await?;
        Ok(reservation)
    }

    async fn update(&self, id: i64, note: String) -> Result<abi::Reservation, abi::Error> {
        let sql = "UPDATE reservations SET note = $1 WHERE id = $2 RETURNING *";
        let reservation: abi::Reservation = sqlx::query_as(sql)
            .bind(note)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(reservation)
    }

    async fn delete(&self, id: i64) -> Result<(), abi::Error> {
        let sql = "DELETE FROM reservations WHERE id = $1";
        sqlx::query(sql).bind(id).execute(&self.pool).await?;
        Ok(())
    }

    async fn get(&self, id: i64) -> Result<abi::Reservation, abi::Error> {
        let sql = "SELECT * FROM reservations WHERE id = $1";
        let reservation: abi::Reservation =
            sqlx::query_as(sql).bind(id).fetch_one(&self.pool).await?;
        Ok(reservation)
    }

    async fn query(
        &self,
        query: abi::ReservationQuery,
    ) -> mpsc::Receiver<Result<abi::Reservation, abi::Error>> {
        let pool = self.pool.clone();
        let (tx, rx) = mpsc::channel(64);

        tokio::spawn(async move {
            let sql = query.to_sql();
            let mut stream = sqlx::query_as(&sql).fetch_many(&pool);
            while let Some(reservation) = stream.next().await {
                match reservation {
                    Ok(Either::Left(reservation)) => {
                        info!("Query result: {:?}", reservation)
                    }
                    Ok(Either::Right(reservation)) => {
                        if tx.send(Ok(reservation)).await.is_err() {
                            // rx is dropped, stop the loop
                            break;
                        }
                    }
                    Err(err) => {
                        warn!("Query error: {:?}", err);
                        if tx.send(Err(err.into())).await.is_err() {
                            // rx is dropped, stop the loop
                            break;
                        }
                    }
                }
            }
        });

        rx
    }

    async fn filter(
        &self,
        _filter: abi::ReservationFilter,
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
    use abi::{
        ReservationConflict, ReservationConflictInfo, ReservationQueryBuilder, ReservationWindow,
    };
    use prost_types::Timestamp;
    use sqlx::PgPool;
    use sqlx_db_tester::TestPg;

    #[tokio::test]
    async fn reserve_should_work_for_valid_window() {
        let db = init_db();
        let pool = db.get_pool().await;
        let (reservation, _store) =
            make_alon_reservation(pool, abi::ReservationStatus::Pending).await;
        assert!(reservation.id > 0);
    }

    #[tokio::test]
    async fn reserve_conflict_reservation_should_reject() {
        let db = init_db();
        let pool = db.get_pool().await;
        let (_r1, s1) = make_alon_reservation(pool.clone(), abi::ReservationStatus::Pending).await;
        let r2 = abi::Reservation::new(
            "alon".to_string(),
            "ocean-view-room-711".to_string(),
            "2022-12-26T15:00:00-0700".parse().unwrap(),
            "2022-12-30T15:00:00-0700".parse().unwrap(),
            "note".to_string(),
            abi::ReservationStatus::Pending,
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

    #[tokio::test]
    async fn confirm_pending_reservation_should_work() {
        let db = init_db();
        let pool = db.get_pool().await;
        let (reservation, store) =
            make_alon_reservation(pool.clone(), abi::ReservationStatus::Pending).await;
        let result = store.confirm(reservation.id).await.unwrap();
        assert_eq!(result.status, abi::ReservationStatus::Confirmed as i32);
        let err = store.confirm(reservation.id).await.unwrap_err();
        assert_eq!(err, abi::Error::NotFound);
    }

    #[tokio::test]
    async fn confirm_confirmed_reservation_should_work() {
        let db = init_db();
        let pool = db.get_pool().await;
        let (reservation, store) =
            make_alon_reservation(pool.clone(), abi::ReservationStatus::Confirmed).await;
        let err = store.confirm(reservation.id).await.unwrap_err();
        assert_eq!(err, abi::Error::NotFound);
    }

    #[tokio::test]
    async fn update_reservation_note_should_work() {
        let db = init_db();
        let pool = db.get_pool().await;
        let (reservation, store) =
            make_alon_reservation(pool.clone(), abi::ReservationStatus::Pending).await;
        let result = store
            .update(reservation.id, "new note".to_string())
            .await
            .unwrap();
        assert_eq!(result.note, "new note");
    }

    #[tokio::test]
    async fn get_reservation_should_work() {
        let db = init_db();
        let pol = db.get_pool().await;
        let (reservation, store) =
            make_alon_reservation(pol.clone(), abi::ReservationStatus::Pending).await;
        let result = store.get(reservation.id).await.unwrap();
        assert_eq!(reservation, result);
    }

    #[tokio::test]
    async fn delete_reservation_should_work() {
        let db = init_db();
        let pool = db.get_pool().await;
        let (reservation, store) =
            make_alon_reservation(pool.clone(), abi::ReservationStatus::Pending).await;
        store.delete(reservation.id).await.unwrap();
        let err = store.get(reservation.id).await.unwrap_err();
        assert_eq!(err, abi::Error::NotFound);
    }

    #[tokio::test]
    async fn query_reservations_should_work() {
        let db = init_db();
        let pool = db.get_pool().await;
        let (reservation, store) =
            make_alon_reservation(pool.clone(), abi::ReservationStatus::Pending).await;
        let query = ReservationQueryBuilder::default()
            .user_id("alon")
            .start("2021-11-01T15:00:00-0700".parse::<Timestamp>().unwrap())
            .end("2023-12-31T12:00:00-0700".parse::<Timestamp>().unwrap())
            .status(abi::ReservationStatus::Pending as i32)
            .build()
            .unwrap();

        let mut rx = store.query(query).await;
        assert_eq!(rx.recv().await, Some(Ok(reservation.clone())));
        assert_eq!(rx.recv().await, None);
    }

    // private none test functions
    fn init_db() -> TestPg {
        TestPg::new(
            "postgres://postgres:123456@localhost:5432".to_string(),
            Path::new("../migrations"),
        )
    }

    async fn make_alon_reservation(
        pool: PgPool,
        status: abi::ReservationStatus,
    ) -> (abi::Reservation, ReservationStore) {
        make_reservation(
            pool,
            "alon",
            "ocean-view-room-711",
            "2022-12-25T15:00:00-0700",
            "2022-12-28T15:00:00-0700",
            "note",
            status,
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
            abi::ReservationStatus::Pending,
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
        status: abi::ReservationStatus,
    ) -> (abi::Reservation, ReservationStore) {
        let store = ReservationStore::new(pool.clone());
        let reservation = abi::Reservation::new(
            uid,
            rid,
            start.parse().unwrap(),
            end.parse().unwrap(),
            note,
            status,
        );
        (store.reserve(reservation).await.unwrap(), store)
    }
}
