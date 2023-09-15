use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use borsh::{BorshDeserialize, BorshSerialize};
use crosstown_bus::{
    CrosstownBus, HandleError, MessageHandler, QueueProperties,
};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct UserCreatedEventMessage {
    pub user_id: String,
    pub user_name: String,
}

pub struct UserCreatedEventHandler {
    received_messages: Arc<Mutex<Vec<UserCreatedEventMessage>>>,
}

impl UserCreatedEventHandler {
    pub fn new(received_messages: Arc<Mutex<Vec<UserCreatedEventMessage>>>) -> Self {
        Self { received_messages }
    }

    pub fn get_received_messages(&self) -> Vec<UserCreatedEventMessage> {
        self.received_messages.lock().unwrap().clone()
    }
}

impl MessageHandler<UserCreatedEventMessage> for UserCreatedEventHandler {
    fn handle(&self, message: Box<UserCreatedEventMessage>) -> Result<(), HandleError> {
        self.received_messages.lock().unwrap().push(*message.clone());

        if message.user_id == "100".to_owned() {
            return Err(HandleError::new("ID 100 rejected".to_owned(), false));
        }
        println!("Message received on User Created Handler: {:?}", message);

        Ok(())
    }

    fn get_handler_action(&self) -> String {
        todo!()
    }
}

#[test]
fn send_receive_successful() -> Result<(), Box<dyn Error>> {
    let received_messages = Arc::new(Mutex::new(Vec::new()));
    let listener = CrosstownBus::new_receiver("amqp://guest:guest@localhost:5672".to_owned())?;

    listener.receive(
        "create_user".to_owned(),
        UserCreatedEventHandler::new(received_messages.clone()),
        QueueProperties {
            auto_delete: false,
            durable: false,
            use_dead_letter: true,
        },
    )?;

    let mut publisher =
        CrosstownBus::new_sender("amqp://guest:guest@localhost:5672".to_owned())?;


    publisher.send(
        "create_user".to_owned(),
        UserCreatedEventMessage {
            user_id: "1234".to_owned(),
            user_name: "Steven Tyler".to_owned(),
        },
    )?;

    publisher.send(
        "create_user".to_owned(),
        UserCreatedEventMessage {
            user_id: "asdf".to_owned(),
            user_name: "Geddy Lee".to_owned(),
        },
    )?;

    publisher.send(
        "create_user".to_owned(),
        UserCreatedEventMessage {
            user_id: "1090".to_owned(),
            user_name: "Geddy Leetre".to_owned(),
        },
    )?;

    publisher.send(
        "create_user".to_owned(),
        UserCreatedEventMessage {
            user_id: "asaaadf".to_owned(),
            user_name: "Geddy Leex".to_owned(),
        },
    )?;

    publisher.send(
        "create_user".to_owned(),
        UserCreatedEventMessage {
            user_id: "asdf".to_owned(),
            user_name: "Geddy Leyyy".to_owned(),
        },
    )?;

    publisher.send(
        "create_user".to_owned(),
        UserCreatedEventMessage {
            user_id: "1000".to_owned(),
            user_name: "Roger Waters".to_owned(),
        },
    )?;

    publisher.send(
        "create_user".to_owned(),
        UserCreatedEventMessage {
            user_id: "1001".to_owned(),
            user_name: "Roger Watersx".to_owned(),
        },
    )?;

    thread::sleep(Duration::from_secs(1));

    let received_messages = received_messages.lock().unwrap();
    println!("Received messages: {:?}", received_messages);

    thread::sleep(Duration::from_secs(1));

    assert_eq!(received_messages.len(), 3);

    assert!(received_messages[0].user_id == "asdf" || received_messages[0].user_id == "1234" || received_messages[0].user_id == "100");

    publisher.close_connection()?;
    Ok(())
}
