use std::{sync::Arc, borrow::BorrowMut, error::Error};

use crosstown_bus::{Bus, EventMessage, MessageHandler};

pub struct MyCustomHandler;

impl MessageHandler::<String> for MyCustomHandler {
    fn handle(&self, message: Box<EventMessage<String>>) -> Result<(), String> {
        println!("Message received: {:?}", message);
        Ok(())
    }
}

#[test]
fn create_subscription() {
    let mut subscriber = Bus::new("amqp://guest:guest@localhost:5672".to_owned()).unwrap();

    _ = subscriber.add_subscription::<String>("abcd".to_owned(),  Arc::new(MyCustomHandler));
    _ = subscriber.publish_event("abcd".to_owned(),EventMessage { id: "1234".to_owned(), payload: "asdfsd".to_owned() } );
    
    let res = subscriber.subscribe_registered_events();
}
