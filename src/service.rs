use crate::{
    generated::{
        hawkeye_collector_server::HawkeyeCollector, OcupationReponse, OcupationRequest,
        OcupationSample,
    },
    storage::{FetchMessage, StorageServiceMessage},
};
use log::info;
use tokio::sync::{mpsc, oneshot};
use tonic::{Request, Response, Status};

#[derive(Debug, Clone)]
pub struct HawkeyeCollectorImpl {
    db_requester: tokio::sync::mpsc::Sender<StorageServiceMessage>,
}

impl HawkeyeCollectorImpl {
    pub fn new(sender: mpsc::Sender<StorageServiceMessage>) -> HawkeyeCollectorImpl {
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
