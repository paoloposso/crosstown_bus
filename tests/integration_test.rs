use core::time::Duration;
use std::{sync::Arc, error::Error, thread};

use crosstown_bus::{Bus, EventMessage, MessageHandler};

pub struct MyCustomHandler;

impl MessageHandler::<String> for MyCustomHandler {
    fn handle(&self, message: Box<EventMessage<String>>) -> Result<(), String> {
        println!("Message received on handler 1: {:?}", message);
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
    let mut publ = Bus::new("amqp://guest:guest@localhost:5672".to_owned())?;

    let _ = futures::executor::block_on(subscriber
        .add_subscription::<String>("queue1".to_owned(),  Arc::new(MyCustomHandler))?
        .add_subscription::<String>("queue1".to_owned(),  Arc::new(MyCustomHandler2))?
        .subscribe_registered_events());
    
    _ = publ.publish_event("queue1".to_owned(),EventMessage { id: "1234".to_owned(), payload: "Hey".to_owned() } );
    _ = publ.publish_event("queue1".to_owned(),EventMessage { id: "1dsds".to_owned(), payload: "Hey".to_owned() } );
    _ = publ.publish_event("queue1".to_owned(),EventMessage { id: "dfsdfsd".to_owned(), payload: "Hey".to_owned() } );
    _ = publ.publish_event("queue1".to_owned(),EventMessage { id: "23dsfd".to_owned(), payload: "Hey".to_owned() } );
    _ = publ.publish_event("queue1".to_owned(),EventMessage { id: "343ggf".to_owned(), payload: "Hey".to_owned() } );
    let _ = thread::sleep(Duration::from_secs(4));
    let _err = publ.close_connection();

    let _ = thread::sleep(Duration::from_secs(60));

    Ok(())
}
