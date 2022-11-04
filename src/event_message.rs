use std::default;

use borsh::{BorshDeserialize, BorshSerialize};

pub trait MessageHandler<T> {
    fn handle(&self, message: Box<T>) -> Result<(), String> where T: Clone + BorshDeserialize + BorshSerialize + 'static;
}