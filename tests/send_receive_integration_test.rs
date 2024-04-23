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
    let subscriber = CrosstownBus::new_subscriber("amqp://guest:guest@localhost:5672".to_owned())?;
    let subscriber2 = CrosstownBus::new_subscriber("amqp://guest:guest@localhost:5672".to_owned())?;

    subscriber.subscribe(
        "user_created".to_owned(),
        NotifyUserHandler::new(received_messages.clone()),
        QueueProperties {
            auto_delete: false,
            durable: false,
            use_dead_letter: true,
            consume_queue_name: Some("queue2".to_string()),
        },
    )?;

    subscriber2.subscribe(
        "user_created".to_owned(),
        AddUserToDBHandler::new(received_messages.clone()),
        QueueProperties {
            auto_delete: false,
            durable: false,
            use_dead_letter: true,
            consume_queue_name: Some("queue1".to_string()),
        },
    )?;

    let mut publisher =
        CrosstownBus::new_publisher("amqp://guest:guest@localhost:5672".to_owned())?;

    publisher.send(
        "user_created".to_owned(),
        UserCreatedMessage {
            user_id: "1234".to_owned(),
            user_name: "Steven Tyler".to_owned(),
            email: "st@test.com".to_owned(),
        },
    )?;

    publisher.send(
        "user_created".to_owned(),
        UserCreatedMessage {
            user_id: "asdf".to_owned(),
            user_name: "Geddy Lee".to_owned(),
            email: "gl@test.com".to_owned(),
        },
    )?;
   
    publisher.send(
        "user_created".to_owned(),
        UserCreatedMessage {
            user_id: "100".to_owned(),
            user_name: "Roger Waters".to_owned(),
            email: "rw@test.com".to_owned(),
        },
    )?;

    thread::sleep(Duration::from_secs(1));

    let received_messages = received_messages.lock().unwrap();

    thread::sleep(Duration::from_secs(1));

    assert!(received_messages.len() == 6);

    publisher.close_connection()?;
    Ok(())
}
