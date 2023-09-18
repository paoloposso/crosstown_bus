use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use borsh::{BorshDeserialize, BorshSerialize};
use crosstown_bus::{
    CrosstownBus, HandleError, MessageHandler, QueueProperties,
};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct NotifyUserMessage {
    pub user_id: String,
    pub user_name: String,
    pub email: String,
}

pub struct NotifyUserHandler {
    received_messages: Arc<Mutex<Vec<NotifyUserMessage>>>,
}

impl NotifyUserHandler {
    pub fn new(received_messages: Arc<Mutex<Vec<NotifyUserMessage>>>) -> Self {
        Self { received_messages }
    }

    pub fn get_received_messages(&self) -> Vec<NotifyUserMessage> {
        self.received_messages.lock().unwrap().clone()
    }
}

impl MessageHandler<NotifyUserMessage> for NotifyUserHandler {
    fn handle(&self, message: Box<NotifyUserMessage>) -> Result<(), HandleError> {
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
    let receiver = CrosstownBus::new_receiver("amqp://guest:guest@localhost:5672".to_owned())?;

    receiver.receive(
        "create_user".to_owned(),
        NotifyUserHandler::new(received_messages.clone()),
        QueueProperties {
            auto_delete: false,
            durable: false,
            use_dead_letter: true,
        },
    )?;

    let mut sender =
        CrosstownBus::new_sender("amqp://guest:guest@localhost:5672".to_owned())?;

    sender.send(
        "create_user".to_owned(),
        NotifyUserMessage {
            user_id: "1234".to_owned(),
            user_name: "Steven Tyler".to_owned(),
            email: "st@test.com".to_owned(),
        },
    )?;

    sender.send(
        "create_user".to_owned(),
        NotifyUserMessage {
            user_id: "asdf".to_owned(),
            user_name: "Geddy Lee".to_owned(),
            email: "gl@test.com".to_owned(),
        },
    )?;
   
    sender.send(
        "create_user".to_owned(),
        NotifyUserMessage {
            user_id: "100".to_owned(),
            user_name: "Roger Waters".to_owned(),
            email: "rw@test.com".to_owned(),
        },
    )?;

    thread::sleep(Duration::from_secs(1));

    let received_messages = received_messages.lock().unwrap();
    println!("Received messages: {:?}", received_messages);

    thread::sleep(Duration::from_secs(1));

    assert!(received_messages.len() == 3 || received_messages.len() == 2);

    assert!(received_messages[0].user_id == "asdf" || received_messages[0].user_id == "1234" || received_messages[0].user_id == "100");

    sender.close_connection()?;
    Ok(())
}
