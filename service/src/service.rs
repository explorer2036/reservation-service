use std::pin::Pin;
use std::task::Poll;

use abi::reservation_service_server::ReservationService as ReservationServiceTrait;
use abi::{
    CancelRequest, CancelResponse, Config, ConfirmRequest, ConfirmResponse, FilterRequest,
    FilterResponse, GetRequest, GetResponse, ListenRequest, QueryRequest, ReserveRequest,
    ReserveResponse, UpdateRequest, UpdateResponse,
};
use futures::Stream;
use reservation::{Reservation, ReservationStore};
use tokio::sync::mpsc;
use tonic::{Response, Status};

use crate::{ReservationService, ReservationStream, TonicReceiverStream};

impl ReservationService {
    pub async fn from_config(config: &Config) -> Result<Self, anyhow::Error> {
        Ok(Self {
            store: ReservationStore::from_config(&config.db).await?,
        })
    }
}

impl<T> TonicReceiverStream<T> {
    pub fn new(inner: mpsc::Receiver<Result<T, abi::Error>>) -> Self {
        Self { inner }
    }
}

impl<T> Stream for TonicReceiverStream<T> {
    type Item = Result<T, Status>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match self.inner.poll_recv(cx) {
            Poll::Ready(Some(Ok(item))) => Poll::Ready(Some(Ok(item))),
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(err.into()))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[tonic::async_trait]
impl ReservationServiceTrait for ReservationService {
    /// make a reservation
    async fn reserve(
        &self,
        request: tonic::Request<ReserveRequest>,
    ) -> Result<tonic::Response<ReserveResponse>, tonic::Status> {
        let request = request.into_inner();
        if request.reservation.is_none() {
            return Err(Status::invalid_argument("missing reservation"));
        }
        let reservation = self.store.reserve(request.reservation.unwrap()).await?;
        Ok(Response::new(ReserveResponse {
            reservation: Some(reservation),
        }))
    }

    /// confirm a pending reservation, if reservation is not pending, do nothing
    async fn confirm(
        &self,
        request: tonic::Request<ConfirmRequest>,
    ) -> Result<tonic::Response<ConfirmResponse>, tonic::Status> {
        let request = request.into_inner();
        let reservation = self.store.confirm(request.id).await?;
        Ok(Response::new(ConfirmResponse {
            reservation: Some(reservation),
        }))
    }

    /// update the reservation note
    async fn update(
        &self,
        request: tonic::Request<UpdateRequest>,
    ) -> Result<tonic::Response<UpdateResponse>, tonic::Status> {
        let request = request.into_inner();
        let reservation = self.store.update(request.id, request.note).await?;
        Ok(Response::new(UpdateResponse {
            reservation: Some(reservation),
        }))
    }

    /// cancel a reservation
    async fn cancel(
        &self,
        request: tonic::Request<CancelRequest>,
    ) -> Result<tonic::Response<CancelResponse>, tonic::Status> {
        let request = request.into_inner();
        let reservation = self.store.delete(request.id).await?;
        Ok(Response::new(CancelResponse {
            reservation: Some(reservation),
        }))
    }

    /// get a reservation by id
    async fn get(
        &self,
        request: tonic::Request<GetRequest>,
    ) -> Result<tonic::Response<GetResponse>, tonic::Status> {
        let request = request.into_inner();
        let reservation = self.store.get(request.id).await?;
        Ok(Response::new(GetResponse {
            reservation: Some(reservation),
        }))
    }

    /// Server streaming response type for the query method.
    type queryStream = ReservationStream;
    /// query reservations by resource id, user id, status, start time, end time
    async fn query(
        &self,
        request: tonic::Request<QueryRequest>,
    ) -> Result<tonic::Response<Self::queryStream>, tonic::Status> {
        let request = request.into_inner();
        if request.query.is_none() {
            return Err(Status::invalid_argument("missing query"));
        }
        let reservations = self.store.query(request.query.unwrap()).await;
        let stream = TonicReceiverStream::new(reservations);
        Ok(Response::new(Box::pin(stream)))
    }

    /// filter reservations, order by reservation id
    async fn filter(
        &self,
        request: tonic::Request<FilterRequest>,
    ) -> Result<tonic::Response<FilterResponse>, tonic::Status> {
        let request = request.into_inner();
        if request.filter.is_none() {
            return Err(Status::invalid_argument("missing filter"));
        }
        let reservations = self.store.filter(request.filter.unwrap()).await?;
        Ok(Response::new(FilterResponse {
            reservations: reservations,
        }))
    }

    /// Server streaming response type for the listen method.
    type listenStream = ReservationStream;
    /// another system could monitor reservation events: added/confirmed/cancelled
    async fn listen(
        &self,
        _request: tonic::Request<ListenRequest>,
    ) -> Result<tonic::Response<Self::listenStream>, tonic::Status> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use abi::{Reservation, ReservationStatus};

    use crate::test_utils::TestConfig;

    use super::*;

    #[tokio::test]
    async fn rpc_reserve_should_work() {
        let config = TestConfig::default();

        let service = ReservationService::from_config(&config).await.unwrap();
        let source = Reservation::new(
            "alon".to_string(),
            "ixia-3230",
            "2022-12-26T15:00:00-0700".parse().unwrap(),
            "2022-12-30T12:00:00-0700".parse().unwrap(),
            "test".to_string(),
            ReservationStatus::Pending,
        );
        let request = tonic::Request::new(ReserveRequest {
            reservation: Some(source.clone()),
        });
        let response = service.reserve(request).await.unwrap();
        let reservation = response.into_inner().reservation;
        assert!(reservation.is_some());

        let reservation = reservation.unwrap();
        assert_eq!(reservation.user_id, source.user_id);
        assert_eq!(reservation.resource_id, source.resource_id);
        assert_eq!(reservation.start, source.start);
        assert_eq!(reservation.end, source.end);
        assert_eq!(reservation.note, source.note);
        assert_eq!(reservation.status, source.status);
    }
}
