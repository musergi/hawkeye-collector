use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hawkeye::generated::{hawkeye_service_client::HawkeyeServiceClient, CpuStatsRequest};
use hawkeye_collector::generated::hawkeye_collector_server::HawkeyeCollectorServer;
use hawkeye_collector::generated::{OcupationReponse, OcupationRequest, OcupationSample};
use hawkeye_collector::storage::SampleStorage;
use hawkeye_collector::Sample;
use hawkeye_collector::{
    generated::hawkeye_collector_server::HawkeyeCollector, storage::memory::MemoryStorage,
};
use log::{info, error};
use tokio::sync::{mpsc, oneshot};
use tokio::time::{self, interval};
use tonic::transport::Server;
use tonic::{Request, Response, Status};

#[derive(Debug)]
enum DatabaseMessage {
    Occupation(f32),
    Fetch(String, oneshot::Sender<Vec<Sample>>),
}

#[derive(Debug, Clone)]
struct HawkeyeCollectorImpl {
    db_requester: tokio::sync::mpsc::Sender<DatabaseMessage>,
}

impl HawkeyeCollectorImpl {
    fn new(sender: mpsc::Sender<DatabaseMessage>) -> HawkeyeCollectorImpl {
        HawkeyeCollectorImpl {
            db_requester: sender,
        }
    }
}

#[tonic::async_trait]
impl HawkeyeCollector for HawkeyeCollectorImpl {
    async fn get_ocupation(
        &self,
        request: Request<OcupationRequest>,
    ) -> Result<tonic::Response<OcupationReponse>, Status> {
        info!("Received GRPC get ocupation request");
        let (tx, rx) = oneshot::channel();
        self.db_requester
            .send(DatabaseMessage::Fetch(
                request.get_ref().identifier.clone(),
                tx,
            ))
            .await
            .unwrap();
        info!("Sent request to database");
        let samples = rx.await.unwrap();
        info!("Received response from database, sending...");
        Ok(Response::new(OcupationReponse {
            samples: samples
                .iter()
                .map(|s| OcupationSample {
                    timestamp: s.timestamp,
                    value: s.value,
                })
                .collect(),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    let mut storage = MemoryStorage::new(10);

    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    let probe_tx = tx.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(10));
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            let peer = "http://shallan:50000";
            let mut client = HawkeyeServiceClient::connect(peer).await.unwrap();
            let response = client
                .get_cpu_stats(CpuStatsRequest { time: 1 })
                .await
                .unwrap();
            log::info!("Ocupation: {}", response.get_ref().ocupation);
            probe_tx
                .send(DatabaseMessage::Occupation(response.get_ref().ocupation))
                .await
                .unwrap();
        }
    });
    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            log::info!("Got: {message:?}");
            match message {
                DatabaseMessage::Occupation(value) => {
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;
                    info!("Timestamp: {:?}", timestamp);
                    storage
                        .store(Sample {
                            identifier: "shallan".to_string(),
                            timestamp,
                            value,
                        })
                        .unwrap();
                }
                DatabaseMessage::Fetch(identifier, sender) => {
                    if let Err(err) = sender.send(storage.fetch(&identifier).unwrap()) {
                        error!("Error sending fetched data {:?}", err)
                    }
                }
            };
        }
        error!("Done receiving due to error");
    });
    Server::builder()
        .add_service(HawkeyeCollectorServer::new(HawkeyeCollectorImpl::new(tx)))
        .serve("127.0.0.1:50001".parse()?)
        .await?;
    Ok(())
}
