use std::{error::Error, sync::Arc};

use borsh::{BorshDeserialize, BorshSerialize};
use crosstown_bus::{MessageHandler, CrosstownBus, HandleError, QueueProperties};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct UserCreatedEventMessage {
    pub user_id: String,
    pub user_name: String
}

pub struct MyCustomHandler;

impl MessageHandler<String> for MyCustomHandler {
    fn handle(&self, message: Box<String>) -> Result<(), HandleError> {
        println!("Message received on handler 1: {:?}", message);
        Ok(())
    }
}

pub struct UserCreatedEventHandler;

impl MessageHandler<UserCreatedEventMessage> for UserCreatedEventHandler {
    fn handle(&self, message: Box<UserCreatedEventMessage>) -> Result<(), HandleError> {
        if message.user_id == "100".to_owned() {
            return Err(HandleError::new("ID 100 rejected".to_owned(), false));
        }
        println!("Message received on User Created Handler: {:?}", message);
        Ok(())
    }
}

#[test]
fn send_receive() -> Result<(), Box<dyn Error>> {
    let subscriber = CrosstownBus::new_queue_listener("amqp://guest:guest@localhost:5672".to_owned())?;

    _ = subscriber.listen("user_created".to_owned(), UserCreatedEventHandler, 
        QueueProperties { auto_delete: false, durable: false, use_dead_letter: true });

    let mut publisher = CrosstownBus::new_queue_publisher("amqp://guest:guest@localhost:5672".to_owned())?;
    _ = publisher.publish_event("user_created".to_owned(), 
        UserCreatedEventMessage {
            user_id: "asdf".to_owned(),
            user_name: "Billy Gibbons".to_owned()
        });

    _ = publisher.publish_event("user_created".to_owned(), 
        UserCreatedEventMessage {
            user_id: "1234".to_owned(),
            user_name: "Dusty Hill".to_owned()
        });

    _ = publisher.publish_event("user_created".to_owned(), 
        UserCreatedEventMessage {
            user_id: "100".to_owned(),
            user_name: "Dusty Hill".to_owned()
        });

    _ = publisher.close_connection();

    Ok(())
}

#[test]
fn broadcast() -> Result<(), Box<dyn Error>> {
    let mut subscriber = CrosstownBus::new_broadcast_subscriber::<UserCreatedEventMessage>("amqp://guest:guest@localhost:5672".to_owned())?;

    _ = subscriber.add_subscription("user_created".to_owned(), Arc::new(UserCreatedEventHandler), 
        QueueProperties { auto_delete: false, durable: false, use_dead_letter: true });

    _ = subscriber.subscribe_registered_events();

    let mut publisher = CrosstownBus::new_broadcast_publisher("amqp://guest:guest@localhost:5672".to_owned())?;
    _ = publisher.publish_event("user_created".to_owned(), 
        UserCreatedEventMessage {
            user_id: "asdf".to_owned(),
            user_name: "Billy Gibbons".to_owned()
        });

    _ = publisher.publish_event("user_created".to_owned(), 
        UserCreatedEventMessage {
            user_id: "1234".to_owned(),
            user_name: "Dusty Hill".to_owned()
        });

    _ = publisher.publish_event("user_created".to_owned(), 
        UserCreatedEventMessage {
            user_id: "100".to_owned(),
            user_name: "Dusty Hill".to_owned()
        });
    Ok(())
}