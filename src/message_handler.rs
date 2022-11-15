use std::error::Error;
use std::fmt;

use borsh::{BorshDeserialize, BorshSerialize};

pub trait MessageHandler<T> {
    fn handle(&self, message: Box<T>) -> Result<(), HandleError> 
        where T: Clone + BorshDeserialize + BorshSerialize + 'static;
}

#[derive(Debug)]
pub struct HandleError {
    details: String,
    pub requeue: bool
}

impl HandleError {
    pub fn new(details: String, requeue: bool) -> Self {
        Self { details, requeue }
    }
}

impl Error for HandleError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl fmt::Display for HandleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}/requeue:{}",self.details, self.requeue)
    }
}
