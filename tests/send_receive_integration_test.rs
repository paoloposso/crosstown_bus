use std::error::Error;

use borsh::{BorshDeserialize, BorshSerialize};
use crosstown_bus::{MessageHandler, CrosstownBus, HandleError, QueueProperties};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct UserCreatedEventMessage {
    pub user_id: String,
    pub user_name: String
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
    let listener = CrosstownBus::new_queue_listener("amqp://guest:guest@localhost:5672".to_owned())?;

    _ = listener.listen("user_created".to_owned(), UserCreatedEventHandler, 
        QueueProperties { auto_delete: false, durable: false, use_dead_letter: true });

    let mut publisher = CrosstownBus::new_queue_publisher("amqp://guest:guest@localhost:5672".to_owned())?;
    _ = publisher.publish_event("user_created".to_owned(), 
        UserCreatedEventMessage {
            user_id: "asdf".to_owned(),
            user_name: "Joe Perry".to_owned()
        });

    _ = publisher.publish_event("user_created".to_owned(), 
        UserCreatedEventMessage {
            user_id: "1234".to_owned(),
            user_name: "Steven Tyler".to_owned()
        });

    _ = publisher.close_connection();

    Ok(())
}
