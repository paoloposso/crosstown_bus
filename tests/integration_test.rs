use std::{sync::Arc, error::Error};

use crosstown_bus::{Bus, EventMessage, MessageHandler};

pub struct MyCustomHandler;

impl MessageHandler::<String> for MyCustomHandler {
    fn handle(&self, message: Box<EventMessage<String>>) -> Result<(), String> {
        println!("Message received on handler 2: {:?}", message);
        Ok(())
    }
}

pub struct MyCustomHandler2;

impl MessageHandler::<String> for MyCustomHandler2 {
    fn handle(&self, message: Box<EventMessage<String>>) -> Result<(), String> {
        println!("Message received on handler 2: {:?}", message);
        Ok(())
    }
}

#[test]
fn create_subscription() -> Result<(), Box<dyn Error>> {
    let subscriber = Bus::new("amqp://guest:guest@localhost:5672".to_owned())?;
    // let publisher = Bus::new("amqp://guest:guest@localhost:5672".to_owned())?;

    _ = subscriber
        .add_subscription::<String>("queue1".to_owned(),  Arc::new(MyCustomHandler))?
        .add_subscription::<String>("queue1".to_owned(),  Arc::new(MyCustomHandler2))?
        .subscribe_registered_events();
    
    // _ = publisher.publish_event("abcd".to_owned(),EventMessage { id: "1234".to_owned(), payload: "asdfsd".to_owned() } );

    Ok(())
}
