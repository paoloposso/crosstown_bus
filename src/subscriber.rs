use std::error::Error;

use amiquip::{Connection, Result, QueueDeclareOptions, ConsumerMessage, ConsumerOptions};

type SubscritpionResult = Result<(), Box<dyn Error>>;

pub struct RabbitSubscriber;

impl RabbitSubscriber {
    pub fn create_subscription(url: String) 
        -> Result<impl Fn(String, fn(message: String) -> SubscritpionResult) -> SubscritpionResult, Box<dyn Error>> {    

        Ok(move |event_name: String, handler: fn(message: String) -> Result<(), Box<dyn Error>>| 
                    -> Result<(), Box<dyn Error>> {
                let mut cnn = Connection::insecure_open(&url)?;
                let channel = cnn.open_channel(None)?;
                let queue = channel.queue_declare(&event_name, QueueDeclareOptions::default())?;
                let consumer = queue.consume(ConsumerOptions::default())?;

                for message in consumer.receiver().iter() {
                    match message {
                        ConsumerMessage::Delivery(delivery) => {
                            let body = String::from_utf8_lossy(&delivery.body);
                            if let Ok(_) = handler(body.to_string()) {
                                consumer.ack(delivery)?;  
                            } else {
                                let _ = consumer.nack(delivery, false);
                            }
                        }
                        other => {
                            println!("Consumer ended: {:?}", other);
                            break;
                        }
                    }
                }
                Ok(())
            })
    }
}