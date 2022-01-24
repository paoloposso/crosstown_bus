use std::{error::Error, thread};

use amiquip::{Connection, QueueDeclareOptions, ConsumerMessage, ConsumerOptions};


type HandleResult = Result<(), Box<dyn Error>>;
type SubscribeResult = Result<(), Box<dyn Error>>;

pub struct RabbitBus {
    url: String
}

impl RabbitBus {
    pub fn new(url: String) -> RabbitBus {
        RabbitBus { url }
    }

    pub fn subscribe(self, event_name: String, handler: fn(message: String) -> HandleResult) -> SubscribeResult {
        let url = self.url;
        let queue = event_name;

        thread::spawn(move || {
            match Connection::insecure_open(&url) {
                Ok(mut cnn) => {
                    let channel = cnn.open_channel(None).unwrap();
                    let queue = channel.queue_declare(queue, QueueDeclareOptions::default()).unwrap();
                    let consumer =  queue.consume(ConsumerOptions::default()).unwrap();
                    
                    for message in consumer.receiver().iter() {
                        match message {
                            ConsumerMessage::Delivery(delivery) => {
                                let body = String::from_utf8_lossy(&delivery.body);
                                let _ = handler(body.to_string());
                            }
                            other => {
                                println!("Consumer ended: {:?}", other);
                                break;
                            }
                        }
                    }
                },
                Err(err) => println!("[bus] Error trying to create connection: {:?}", err),
            }
        });

        Ok(())
    }
}
