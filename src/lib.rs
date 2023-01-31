#[derive(Debug, Clone)]
pub struct Sample {
    pub identifier: String,
    pub timestamp: u64,
    pub value: f32,
}

pub mod config;
pub mod polling;
pub mod service;
pub mod storage;

pub mod generated {
    tonic::include_proto!("hawkeye_collector");
}
