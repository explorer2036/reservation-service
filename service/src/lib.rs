mod service;
mod test_utils;

use abi::{reservation_service_server::ReservationServiceServer, Config, Reservation};
use futures::Stream;
use reservation::ReservationStore;
use std::pin::Pin;
use tokio::sync::mpsc;
use tonic::{transport::Server, Status};

pub struct ReservationService {
    store: ReservationStore,
}

pub struct TonicReceiverStream<T> {
    inner: mpsc::Receiver<Result<T, abi::Error>>,
}

type ReservationStream = Pin<Box<dyn Stream<Item = Result<Reservation, Status>> + Send>>;

pub async fn start_server(config: &Config) -> Result<(), anyhow::Error> {
    let addr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    let service = ReservationService::from_config(config).await?;
    let service = ReservationServiceServer::new(service);

    println!("Listening on {}", addr);
    Server::builder().add_service(service).serve(addr).await?;
    Ok(())
}
