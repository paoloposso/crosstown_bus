#[cfg(test)]

use core::time;
use std::{thread, rc::Rc};

use borsh::{BorshSerialize, BorshDeserialize};

use crosstown_bus::{Publisher, Subscriber, Message, MessageHandler};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct UserUpdated(String, String);

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct UserCreated {
    name: String,
    id: String
}

pub struct MyCustomHandler;

impl MessageHandler::<String> for MyCustomHandler {
    fn handle(&self, message: Box<Message<String>>) -> Result<(), String> {
        println!("Message received: {:?}", message);
        Ok(())
    }
}

#[test]
fn create_subscription() {
    let mut subscriber = Subscriber::new("amqp://guest:guest@localhost:5672".to_string());

    _ = subscriber.add_subscription::<String>("test1".to_owned(), Rc::new(MyCustomHandler {}));
    _ = subscriber.add_subscription::<String>("test1".to_owned(), Rc::new(MyCustomHandler {}));

    let publ = Publisher::new("amqp://guest:guest@localhost:5672".to_string()).unwrap();
    // publ.publish_event();

    futures::executor::block_on(subscriber.subscribe_registered_events());
}
