use super::{SampleStorage, StorageError};
use crate::Sample;
use log::info;
use std::collections::VecDeque;

/// Storage implementation that holds the passed samples in memory
pub struct MemoryStorage {
    max_size: usize,
    content: VecDeque<Sample>,
}

impl MemoryStorage {
    /// Constructor for the memory storage
    ///
    /// * `size` - Number of messages to hold in memory, any message beyond this count
    ///            will replace the oldest message in memory
    pub fn new(size: usize) -> MemoryStorage {
        MemoryStorage {
            max_size: size,
            content: VecDeque::with_capacity(size),
        }
    }
}

impl SampleStorage for MemoryStorage {
    fn store(&mut self, sample: Sample) -> Result<(), StorageError> {
        while self.content.len() >= self.max_size {
            self.content.pop_front();
        }
        self.content.push_back(sample);
        info!("Added sample, current buffer {}", self.content.len());
        Ok(())
    }

    fn fetch(&self, identifier: &str) -> Result<Vec<Sample>, StorageError> {
        Ok(self
            .content
            .iter()
            .filter(|sample| sample.identifier == identifier)
            .cloned()
            .collect())
    }
}
