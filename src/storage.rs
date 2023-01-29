use crate::Sample;
use log::error;
use std::{error::Error, fmt};
use tokio::sync::{mpsc, oneshot};

pub struct StorageService<S> {
    receiver: mpsc::Receiver<StorageServiceMessage>,
    storage: S,
}

impl<S> StorageService<S> {
    pub fn new(receiver: mpsc::Receiver<StorageServiceMessage>, storage: S) -> StorageService<S> {
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
        match message {
            StorageServiceMessage::Store(sample) => self.storage.store(sample).unwrap(),
            StorageServiceMessage::Fetch(message) => {
                let FetchMessage { identifier, sender } = message;
                let response = self.storage.fetch(&identifier).unwrap();
                sender.send(response).unwrap();
            }
        }
    }
}

#[derive(Debug)]
pub enum StorageServiceMessage {
    Store(Sample),
    Fetch(FetchMessage),
}

#[derive(Debug)]
pub struct FetchMessage {
    identifier: String,
    sender: oneshot::Sender<Vec<Sample>>,
}

impl FetchMessage {
    pub fn new(identifier: String, sender: oneshot::Sender<Vec<Sample>>) -> FetchMessage {
        FetchMessage { identifier, sender }
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
