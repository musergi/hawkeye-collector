use hawkeye::generated::hawkeye_service_client::HawkeyeServiceClient;
use hawkeye_collector::generated::hawkeye_collector_server::HawkeyeCollectorServer;
use hawkeye_collector::generated::{OcupationReponse, OcupationRequest, OcupationSample};
use hawkeye_collector::polling::PollingService;
use hawkeye_collector::storage::{FetchMessage, StorageService, StorageServiceMessage};
use hawkeye_collector::{
    generated::hawkeye_collector_server::HawkeyeCollector, storage::memory::MemoryStorage,
};
use log::info;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::time::interval;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

#[derive(Debug, Clone)]
struct HawkeyeCollectorImpl {
    db_requester: tokio::sync::mpsc::Sender<StorageServiceMessage>,
}

impl HawkeyeCollectorImpl {
    fn new(sender: mpsc::Sender<StorageServiceMessage>) -> HawkeyeCollectorImpl {
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
        let request = FetchMessage::new(request.get_ref().identifier.clone(), tx);
        self.db_requester
            .send(StorageServiceMessage::Fetch(request))
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

    let (tx, rx) = mpsc::channel(100);
    let storage = MemoryStorage::new(10);
    let storage_service = StorageService::new(rx, storage);
    tokio::spawn(storage_service.run());

    let peer = "http://shallan:50000";
    let client = HawkeyeServiceClient::connect(peer).await.unwrap();
    let polling_service =
        PollingService::new(interval(Duration::from_secs(10)), tx.clone(), client);
    tokio::spawn(polling_service.run());

    Server::builder()
        .add_service(HawkeyeCollectorServer::new(HawkeyeCollectorImpl::new(tx)))
        .serve("127.0.0.1:50001".parse()?)
        .await?;

    Ok(())
}
