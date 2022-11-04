use core::time::Duration;
use std::{sync::Arc, error::Error, thread};

use borsh::{BorshDeserialize, BorshSerialize};
use crosstown_bus::{QueueBus, MessageHandler};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct UserCreatedEventMessage {
    pub user_id: String,
    pub user_name: String,
    pub timestamp: u64
}

pub struct MyCustomHandler;

impl MessageHandler<String> for MyCustomHandler {
    fn handle(&self, message: Box<String>) -> Result<(), String> {
        println!("Message received on handler 1: {:?}", message);
        Ok(())
    }
}

pub struct UserCreatedEventHandler;

impl MessageHandler<UserCreatedEventMessage> for UserCreatedEventHandler {
    fn handle(&self, message: Box<UserCreatedEventMessage>) -> Result<(), String> {
        println!("Message received on User Created Handler: {:?}", message);
        Ok(())
    }
}

#[test]
fn create_subscription() -> Result<(), Box<dyn Error>> {
    let subscriber = QueueBus::<UserCreatedEventMessage>::new("amqp://guest:guest@localhost:5672".to_owned())?;
    let mut publ = QueueBus::<String>::new("amqp://guest:guest@localhost:5672".to_owned())?;

    let _ = futures::executor::block_on(
        subscriber
            .add_subscription::<UserCreatedEventMessage>("queue3".to_owned(),  Arc::new(UserCreatedEventHandler))?
            .add_subscription::<UserCreatedEventMessage>("queue4".to_owned(),  Arc::new(UserCreatedEventHandler))?
            .subscribe_registered_events()
    );
    
    _ = publ.publish_event("queue3".to_owned(), UserCreatedEventMessage { user_id: "11111".to_owned(), user_name: "paolo".to_owned(), timestamp: 3943043274 });
    _ = publ.publish_event("queue4".to_owned(), UserCreatedEventMessage { user_id: "22222".to_owned(), user_name: "paolo".to_owned(), timestamp: 3943043274 });
    
    let _ = thread::sleep(Duration::from_secs(4));
    let _err = publ.close_connection();

    let _ = thread::sleep(Duration::from_secs(60));

    Ok(())
}
