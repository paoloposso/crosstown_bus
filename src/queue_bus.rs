use std::{cell::RefCell, error::Error, sync::{Mutex, Arc}, thread};

use amiquip::{Connection, Publish, ConsumerOptions, ConsumerMessage, QueueDeclareOptions};
use borsh::{BorshSerialize, BorshDeserialize};

use crate::MessageHandler;

pub type GenericResult = Result<(), Box<dyn Error>>;

#[derive(Default)]
pub struct QueueProperties {
    /// queue will be deleted from RabbitMQ when the last consumer disconnects
    pub auto_delete: bool,
    /// if set to true, queue will survive when server restarts
    pub durable: bool,
}

pub struct QueuePublisher {
    pub(crate) cnn: RefCell<Connection>
}

pub struct QueueSubscriber {
    pub(crate) cnn: RefCell<Connection>
}

impl QueueSubscriber {
    pub async fn subscribe_event<T>(self, event_name: String, 
        event_handler: impl MessageHandler<T> + Send + Sync + 'static,
        queue_properties: Option<QueueProperties>) -> GenericResult
        where T : BorshDeserialize + BorshSerialize + Clone + 'static {

        let connection = Arc::new(Mutex::new(self.cnn));
        let handler = Arc::new(event_handler);
        let queue_name = event_name.clone();
        let cnn = Arc::clone(&connection);

        thread::spawn(move || {
            let channel = cnn.lock().unwrap().get_mut().open_channel(None).unwrap();

            let mut queue_options = QueueDeclareOptions::default();
            if let Some(props) = queue_properties {
                queue_options.auto_delete = props.auto_delete;
                queue_options.durable = props.durable;
            }

            match channel.queue_declare(queue_name, queue_options) {
                Ok(queue) => {
                    match queue.consume(ConsumerOptions::default()) {
                        Ok(consumer) => {
                            for message in consumer.receiver().iter() {
                                match message {
                                    ConsumerMessage::Delivery(delivery) => {
                                        let str_message = String::from_utf8_lossy(&delivery.body).to_string();
                                        let mut buf = str_message.as_bytes();
    
                                        if let Ok(model) = BorshDeserialize::deserialize(&mut buf) {
                                            let handle_result = handler.handle(model);
                                            _ = delivery.ack(&channel);
                                        } else {
                                            _ = delivery.nack(&channel, false);
                                            eprintln!("[crosstown_bus] Error trying to desserialize. Check message format. Message: {:?}", str_message);
                                        }
                                    }
                                    other => {
                                        eprintln!("[crosstown_bus] Consumer ended: {:?}", other);
                                        break;
                                    }
                                }
                            }
                        }
                        Err(err) => eprintln!("[crosstown_bus] Error trying to consume: {:?}", err),
                    };
                },
                Err(err) => eprintln!("[crosstown_bus] Error trying to consume: {:?}", err),
            } 
        });
        Ok(())
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