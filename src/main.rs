use hawkeye::generated::hawkeye_service_client::HawkeyeServiceClient;
use hawkeye_collector::config::Config;
use hawkeye_collector::generated::hawkeye_collector_server::HawkeyeCollectorServer;
use hawkeye_collector::polling::PollingService;
use hawkeye_collector::service::HawkeyeCollectorImpl;
use hawkeye_collector::storage::memory::MemoryStorage;
use hawkeye_collector::storage::StorageService;
use std::time::Duration;
use std::{env, fs};
use tokio::sync::mpsc;
use tokio::time::interval;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    let args = env::args().collect::<Vec<_>>();
    let config_path = args.get(1).expect("Expected one argument with config path");
    let config: Config =
        serde_json::from_str(&fs::read_to_string(config_path).expect("Config file not found"))
            .expect("Invalid config");

    let (tx, rx) = mpsc::channel(config.message_channel_size);
    let storage = MemoryStorage::new(config.memory_storage_size);
    let storage_service = StorageService::new(rx, storage);
    tokio::spawn(storage_service.run());

    for peer in config.peers {
        let client = HawkeyeServiceClient::connect(peer.peer_address)
            .await
            .unwrap();
        let polling_service = PollingService::new(
            interval(Duration::from_secs(peer.polling_interval)),
            tx.clone(),
            client,
        );
        tokio::spawn(polling_service.run());
    }

    let api_tx = tx.clone();
    tokio::spawn(async move {
        warp::serve(hawkeye_collector::rest::routes(api_tx))
            .run(config.rest_address)
            .await
    });

    Server::builder()
        .add_service(HawkeyeCollectorServer::new(HawkeyeCollectorImpl::new(tx)))
        .serve(config.grpc_address)
        .await?;

    Ok(())
}
