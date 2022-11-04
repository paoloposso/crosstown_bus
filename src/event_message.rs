use std::default;

use borsh::{BorshDeserialize, BorshSerialize};

pub trait MessageHandler<T> {
    fn handle(&self, message: Box<T>) -> Result<(), String> where T: Clone + BorshDeserialize + BorshSerialize + 'static;
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct IntegrationEventMessage {
    pub event_id: String,
    pub payload: String,
    pub timestamp: u64
}