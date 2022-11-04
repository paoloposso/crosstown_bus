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

pub struct MyCustomHandler2;

impl MessageHandler<UserCreatedEventMessage> for MyCustomHandler2 {
    fn handle(&self, message: Box<UserCreatedEventMessage>) -> Result<(), String> {
        println!("Message received on handler 2: {:?}", message);
        Ok(())
    }
}

#[test]
fn create_subscription() -> Result<(), Box<dyn Error>> {
    let subscriber = QueueBus::<String>::new("amqp://guest:guest@localhost:5672".to_owned())?;
    let subscriber2 = QueueBus::<UserCreatedEventMessage>::new("amqp://guest:guest@localhost:5672".to_owned())?;
    let mut publ = QueueBus::<String>::new("amqp://guest:guest@localhost:5672".to_owned())?;

    let _ = futures::executor::block_on(subscriber
        .add_subscription::<String>("queue1".to_owned(),  Arc::new(MyCustomHandler))?
        .add_subscription::<String>("queue2".to_owned(),  Arc::new(MyCustomHandler))?
        .subscribe_registered_events());
    
        _ = publ.publish_event("queue1".to_owned(), "123456".to_owned());
    _ = publ.publish_event("queue1".to_owned(), "123456".to_owned());
    _ = publ.publish_event("queue1".to_owned(), "123456".to_owned());
    _ = publ.publish_event("queue2".to_owned(), "queue2 aaaaa".to_owned());
    
    
    let _ = thread::sleep(Duration::from_secs(4));
    let _err = publ.close_connection();

    let _ = thread::sleep(Duration::from_secs(60));

    Ok(())
}
