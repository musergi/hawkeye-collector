use std::time::{SystemTime, UNIX_EPOCH};

use crate::{storage::StorageServiceMessage, Sample};
use hawkeye::generated::{
    hawkeye_service_client::HawkeyeServiceClient, CpuStatsRequest, CpuStatsResponse,
};
use log::warn;
use tokio::{sync::mpsc, time::Interval};
use tonic::{transport::Channel, Response};

pub struct PollingService {
    interval: Interval,
    sender: mpsc::Sender<StorageServiceMessage>,
    client: HawkeyeServiceClient<Channel>,
}

impl PollingService {
    pub fn new(
        interval: Interval,
        sender: mpsc::Sender<StorageServiceMessage>,
        client: HawkeyeServiceClient<Channel>,
    ) -> PollingService {
        PollingService {
            interval,
            sender,
            client,
        }
    }

    pub async fn run(mut self) {
        loop {
            self.interval.tick().await;
            match self.client.get_cpu_stats(CpuStatsRequest { time: 1 }).await {
                Ok(response) => self.handle_response(&response).await,
                Err(err) => warn!("Error occured when polling: {err:?}"),
            }
        }
    }

    async fn handle_response(&self, response: &Response<CpuStatsResponse>) {
        let CpuStatsResponse { ocupation } = response.get_ref();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        self.sender
            .send(StorageServiceMessage::Store(Sample {
                identifier: "shallan".to_string(),
                timestamp,
                value: *ocupation,
            }))
            .await
            .unwrap();
    }
}
