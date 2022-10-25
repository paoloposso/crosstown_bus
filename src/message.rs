use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct Message<T> {
    // timestamp: i64,
    pub id: String,
    pub data: T
}

pub trait MessageHandler<T> {
    fn handle(&self, message: Box<Message<T>>) -> Result<(), String>;
}