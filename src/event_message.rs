use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct EventMessage<T> {
    // timestamp: i64,
    pub id: String,
    pub payload: T,
}

pub trait MessageHandler<T> {
    fn handle(&self, message: Box<EventMessage<T>>) -> Result<(), String>;
}