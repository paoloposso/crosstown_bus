use std::sync::{Arc, Mutex};

use borsh::{BorshDeserialize, BorshSerialize};
use crosstown_bus::{HandleError, MessageHandler};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct UserCreatedMessage {
    pub user_id: String,
    pub user_name: String,
    pub email: String,
}

pub struct NotifyUserHandler {
    received_messages: Arc<Mutex<Vec<UserCreatedMessage>>>,
}

impl NotifyUserHandler {
    pub fn new(received_messages: Arc<Mutex<Vec<UserCreatedMessage>>>) -> Self {
        Self { received_messages }
    }

    pub fn get_received_messages(&self) -> Vec<UserCreatedMessage> {
        self.received_messages.lock().unwrap().clone()
    }
}

impl MessageHandler<UserCreatedMessage> for NotifyUserHandler {
    fn handle(&self, message: Box<UserCreatedMessage>) -> Result<(), HandleError> {
        self.received_messages.lock().unwrap().push(*message.clone());

        if message.user_id == "100".to_owned() {
            return Err(HandleError::new("ID 100 rejected".to_owned(), false));
        }
        println!("Message received on User Created Handler: {:?}", message);

        Ok(())
    }
}

pub struct AddUserToDBHandler {
    received_messages: Arc<Mutex<Vec<UserCreatedMessage>>>,
}

impl AddUserToDBHandler {
    pub fn new(received_messages: Arc<Mutex<Vec<UserCreatedMessage>>>) -> Self {
        Self { received_messages }
    }

    pub fn get_received_messages(&self) -> Vec<UserCreatedMessage> {
        self.received_messages.lock().unwrap().clone()
    }
}

impl MessageHandler<UserCreatedMessage> for AddUserToDBHandler {
    fn handle(&self, message: Box<UserCreatedMessage>) -> Result<(), HandleError> {
        self.received_messages.lock().unwrap().push(*message.clone());

        if message.user_id == "100".to_owned() {
            return Err(HandleError::new("ID 100 rejected".to_owned(), false));
        }
        println!("Message received on User Created Handler: {:?}", message);

        Ok(())
    }
}