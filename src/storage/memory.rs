use std::collections::VecDeque;

use log::info;

use crate::Sample;

use super::SampleStorage;

pub struct MemoryStorage {
    max_size: usize,
    content: VecDeque<Sample>,
}

impl MemoryStorage {
    pub fn new(size: usize) -> MemoryStorage {
        MemoryStorage {
            max_size: size,
            content: VecDeque::with_capacity(size),
        }
    }
}

impl SampleStorage for MemoryStorage {
    fn store(&mut self, sample: Sample) -> Result<(), super::StorageError> {
        while self.content.len() >= self.max_size {
            self.content.pop_front();
        }
        self.content.push_back(sample);
        info!("Added sample, current buffer {}", self.content.len());
        Ok(())
    }

    fn fetch(&self, identifier: &str) -> Result<Vec<Sample>, super::StorageError> {
        Ok(self
            .content
            .iter()
            .filter(|sample| sample.identifier == identifier)
            .cloned()
            .collect())
    }
}
