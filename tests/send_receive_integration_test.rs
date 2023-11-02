use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crosstown_bus::{
    CrosstownBus, QueueProperties,
};

use crate::events::{NotifyUserHandler, AddUserToDBHandler, UserCreatedMessage};

mod events;

#[test]
fn send_receive_successful() -> Result<(), Box<dyn Error>> {
    let received_messages = Arc::new(Mutex::new(Vec::new()));
    let receiver = CrosstownBus::new_receiver("amqp://guest:guest@localhost:5672".to_owned())?;
    let receiver2 = CrosstownBus::new_receiver("amqp://guest:guest@localhost:5672".to_owned())?;

    receiver.receive(
        "user_created".to_owned(),
        NotifyUserHandler::new(received_messages.clone()),
        QueueProperties {
            auto_delete: false,
            durable: false,
            use_dead_letter: true,
            consume_queue_name: Some("queue2".to_string()),
        },
    )?;

    receiver2.receive(
        "create_user".to_owned(),
        AddUserToDBHandler::new(received_messages.clone()),
        QueueProperties {
            auto_delete: false,
            durable: false,
            use_dead_letter: true,
            consume_queue_name: Some("queue1".to_string()),
        },
    )?;

    let mut sender =
        CrosstownBus::new_sender("amqp://guest:guest@localhost:5672".to_owned())?;

    sender.send(
        "user_created".to_owned(),
        UserCreatedMessage {
            user_id: "1234".to_owned(),
            user_name: "Steven Tyler".to_owned(),
            email: "st@test.com".to_owned(),
        },
    )?;

    sender.send(
        "user_created".to_owned(),
        UserCreatedMessage {
            user_id: "asdf".to_owned(),
            user_name: "Geddy Lee".to_owned(),
            email: "gl@test.com".to_owned(),
        },
    )?;
   
    sender.send(
        "user_created".to_owned(),
        UserCreatedMessage {
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
