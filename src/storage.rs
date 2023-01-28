use std::{
    error::Error,
    fmt::{self, Display},
};

use crate::Sample;

#[derive(Debug)]
pub struct StorageError(String);

impl Display for StorageError {
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
