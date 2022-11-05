use std::{cell::{Cell, RefCell}, error::Error, sync::{Mutex, Arc}, thread};

use amiquip::{Connection, Publish, ConsumerOptions, ConsumerMessage, QueueDeclareOptions, Queue};
use borsh::{BorshSerialize, BorshDeserialize};
use tokio::runtime::Handle;

use crate::MessageHandler;

pub type GenericResult = Result<(), Box<dyn Error>>;

pub struct QueuePublisher {
    pub(crate) cnn: RefCell<Connection>
}

pub struct QueueSubscriber {
    pub(crate) cnn: RefCell<Connection>
}

impl QueueSubscriber {
    pub async fn subscribe_event<T>(self, event_name: String, 
        event_handler: impl MessageHandler<T> + Send + Sync + 'static)
        where T : BorshDeserialize + BorshSerialize + Clone + 'static {
        let connection = Arc::new(Mutex::new(self.cnn));
        let handler = Arc::new(event_handler);
        let queue_name = event_name.clone();
        let cnn = Arc::clone(&connection);
        thread::spawn(move || {
            let channel = cnn.lock().unwrap().get_mut().open_channel(None).unwrap();
            let queue: Queue = channel.queue_declare(queue_name, QueueDeclareOptions {
                durable: false,
                exclusive: false,
                auto_delete: false,
                ..Default::default()
            }).unwrap();
            match queue.consume(ConsumerOptions::default()) {
                Ok(consumer) => {
                    for message in consumer.receiver().iter() {
                        match message {
                            ConsumerMessage::Delivery(delivery) => {
                                let str_message = String::from_utf8_lossy(&delivery.body).to_string();
                                let mut buf = str_message.as_bytes();

                                if let Ok(model) = BorshDeserialize::deserialize(&mut buf) {
                                    _ = handler.handle(model);
                                    _ = delivery.ack(&channel);
                                } else {
                                    _ = delivery.nack(&channel, false);
                                    eprintln!("[bus] Error trying to desserialize. Check message format. Message: {:?}", str_message);
                                }
                            }
                            other => {
                                println!("Consumer ended: {:?}", other);
                                break;
                            }
                        }
                    }
                }
                Err(_) => {
                    eprintln!("[bus] Error trying to consume");
                }
            };
        });
    }
    pub fn close_connection(self) -> GenericResult {
        self.cnn.into_inner().close()?;
        Ok(())
    }
}


impl QueuePublisher {
    pub fn publish_event<T>(&mut self, event_name: String, message: T)
        -> GenericResult 
            where T: BorshSerialize + BorshDeserialize {
        let mut buffer = Vec::new();
        message.serialize(&mut buffer)?;
        if let Ok(channel) = self.cnn.get_mut().open_channel(None) {
            let publish_result = channel.basic_publish::<String>(
                "".to_owned(),
                Publish {
                    body: &buffer,
                    routing_key: event_name.to_owned(),
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

    pub fn close_connection(self) -> GenericResult {
        self.cnn.into_inner().close()?;
        Ok(())
    }
}