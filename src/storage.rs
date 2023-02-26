use crate::Sample;
use log::{debug, error};
use std::{error::Error, fmt};

type RequestChannel = tokio::sync::mpsc::Receiver<StorageServiceMessage>;

pub struct StorageService<S> {
    receiver: RequestChannel,
    storage: S,
}

impl<S> StorageService<S> {
    pub fn new(receiver: RequestChannel, storage: S) -> StorageService<S> {
        StorageService { receiver, storage }
    }

    pub async fn run(mut self)
    where
        S: SampleStorage,
    {
        while let Some(message) = self.receiver.recv().await {
            self.handle_message(message);
        }
        error!("All senders to storage closed.")
    }

    fn handle_message(&mut self, message: StorageServiceMessage)
    where
        S: SampleStorage,
    {
        debug!("Storage request: {}", message);
        match message {
            StorageServiceMessage::Store(sample) => self.storage.store(sample).unwrap(),
            StorageServiceMessage::Fetch {
                identifier,
                response_channel,
            } => {
                let response = self.storage.fetch(&identifier).unwrap();
                response_channel.send(response).unwrap();
            }
        }
    }
}

#[derive(Debug)]
pub enum StorageServiceMessage {
    Store(Sample),
    Fetch {
        identifier: String,
        response_channel: tokio::sync::oneshot::Sender<Vec<Sample>>,
    },
}

impl fmt::Display for StorageServiceMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StorageServiceMessage::Store(sample) => {
                write!(f, "Store({}, {})", sample.timestamp, sample.identifier)
            }
            StorageServiceMessage::Fetch { identifier, .. } => {
                write!(f, "Fetch({})", identifier)
            }
        }
    }
}

#[derive(Debug)]
pub struct StorageError(String);

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for StorageError {}

pub trait SampleStorage {
    fn store(&mut self, sample: Sample) -> Result<(), StorageError>;
    fn fetch(&self, identifier: &str) -> Result<Vec<Sample>, StorageError>;
}

pub mod memory;
