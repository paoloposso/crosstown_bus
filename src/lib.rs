//! # Rusty Bus
//!
//! `rusty_bus` is an easy-to-use event bus in Rust with RabbitMQ.

use std::{error::Error, thread};

use amiquip::{Connection, QueueDeclareOptions, ConsumerMessage, ConsumerOptions, FieldTable, ExchangeDeclareOptions, Publish};

type HandleResult = Result<(), Box<dyn Error>>;
type SubscribeResult = Result<(), Box<dyn Error>>;
type PublishResult = Result<(), Box<dyn Error>>;

pub struct Bus {
    url: String
}

pub fn new_rabbit_bus(url: String) -> Bus {
    Bus { url }
}

impl Bus {
    pub fn publish_event<'a>(&'a self, event_name: String, message: String) -> PublishResult {
        let url = self.url.to_owned();
        let event = event_name.to_owned();
        if let Ok(channel) = Connection::insecure_open(&url)?.open_channel(None) {
            let publish_result = channel.basic_publish::<String>(event_name, Publish {
                body: message.as_bytes(),
                routing_key: event,
                mandatory: false,
                immediate: false,
                properties: Default::default(),
            });
            if publish_result.is_err() { 
                return Err(Box::new(publish_result.unwrap_err()));
            }
        }
        Ok(())
    }

    pub fn subscribe_event<'a>(&'a self, event_name: String, handler: fn(message: String) -> HandleResult) -> SubscribeResult {
        let url = self.url.to_owned();
        let queue_name = event_name;

        thread::spawn(move || {
            match Connection::insecure_open(&url) {
                Ok(mut cnn) => {
                    if let Ok(channel) = cnn.open_channel(None) {
                        match channel.queue_declare(queue_name.to_owned(), QueueDeclareOptions {
                            durable: false,
                            exclusive: false,
                            auto_delete: false,
                            ..Default::default()
                        }) {
                            Ok(queue) => {
                                let exchange_declare_options = ExchangeDeclareOptions {
                                    auto_delete: false,
                                    durable: false,
                                    internal: false,
                                    ..Default::default()
                                };
            
                                let _ = queue.bind(
                                    &channel.exchange_declare::<String>(amiquip::ExchangeType::Fanout, queue_name.to_owned(), exchange_declare_options).unwrap(), 
                                    "".to_string(), FieldTable::new());
            
                                if let Ok(consumer) = queue.consume(ConsumerOptions::default()) {
                                    for message in consumer.receiver().iter() {
                                        match message {
                                            ConsumerMessage::Delivery(delivery) => {
                                                let body = String::from_utf8_lossy(&delivery.body);
                
                                                let _ = match handler(body.to_string()) {
                                                    Ok(_) => { 
                                                        let _ = delivery.ack(&channel);
                                                    },
                                                    Err(error) => { 
                                                        println!("Error trying to process message: {:?}", error.as_ref());
                                                        let _ = delivery.nack(&channel, false);
                                                    },
                                                };
                                            }
                                            other => {
                                                println!("Consumer ended: {:?}", other);
                                                break;
                                            }
                                        }
                                    }
                                } else {
                                    eprintln!("[bus] Error trying to consume");
                                }
                            },
                            Err(err) => eprintln!("[bus] Error creating Queue: {:?}", err),
                        };
                    } else {
                        eprintln!("[bus] Error opening channel");
                    }
                },
                Err(err) => eprintln!("[bus] Error trying to create connection: {:?}", err),
            }
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{new_rabbit_bus};

    #[test]
    fn new_bus_works() {
        let bus = new_rabbit_bus("url".to_string());
        if bus.url.len() > 0 {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
