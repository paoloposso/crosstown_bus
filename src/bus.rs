use amiquip::{ConsumerOptions, ConsumerMessage, QueueDeclareOptions, 
    Connection, Publish, Queue};
use borsh::{BorshDeserialize, BorshSerialize};
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::ops::{DerefMut, Deref};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::EventMessage;
use crate::event_message::MessageHandler;

// #[derive(Clone)]
pub struct Bus {
    cnn: Arc<Mutex<Connection>>,
    subs_manager: SubscriptionManager
}

#[derive(Clone)]
pub struct SubscriptionManager {
    pub handlers: HashMap<String, Arc<dyn MessageHandler<String> + Send + Sync>>
}

impl Bus {
    pub fn new(url: String) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            cnn: Arc::new(Mutex::new(Connection::insecure_open(&url)?)),
            subs_manager: SubscriptionManager { handlers: HashMap::new() }
        })
    }

    pub fn add_subscription<T>(mut self, event_name: String, 
        handler: Arc<dyn MessageHandler<String> + Send + Sync>
    ) 
        -> Result<Self, Box<dyn Error>> where T: ?Sized {
        if self.subs_manager.handlers.contains_key(&event_name) {
            // self.subs_manager.handlers.get_mut(&event_name).unwrap() = &mut handler;
        } else {
            // let mut handler_list = Vec::new();
            // handler_list.push(handler);
            self.subs_manager.handlers.insert(event_name, handler.clone());
        }
        Ok(self)
    }

    pub fn publish_event<T>(&mut self, event_name: String, message: EventMessage::<T>) 
        -> Result<(), Box<dyn Error>> where T: BorshSerialize + BorshDeserialize {
        let mut buffer = Vec::new();
        message.serialize(&mut buffer)?;

        if let Ok(channel) = self.cnn.lock().unwrap().open_channel(None) {
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

    pub async fn subscribe_registered_events(self) -> Result<(), Box::<dyn Error>> {
        let handlers = self.subs_manager.handlers;
        for (event_name, handlers_list) in handlers {
            let handler = handlers_list;
            // for handler in handlers_list 

            let event = event_name.clone();
            let connection = Arc::clone(&self.cnn);

            tokio::spawn(async move {
                let queue_name = event;
                let channel = connection.lock().unwrap().open_channel(None).unwrap();
                let queue: Queue = channel.queue_declare(queue_name.to_owned(), QueueDeclareOptions {
                    durable: false,
                    exclusive: false,
                    auto_delete: true,
                    ..Default::default()
                }).unwrap();
                
                match queue.borrow().consume(ConsumerOptions::default()) {
                    Ok(consumer) => {
                        for message in consumer.receiver().iter() {
                            match message {
                                ConsumerMessage::Delivery(delivery) => {
                                    let str_message = String::from_utf8_lossy(&delivery.body).to_string();
                                    let mut buf = str_message.as_bytes();
    
                                    if let Ok(model) = BorshDeserialize::deserialize(&mut buf) {
                                        _ = handler.handle(model);
                                        // _ = delivery.ack(&channel);
                                    } else {
                                        // _ = delivery.nack(&channel, false);
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
        Ok(())
    }

    // pub fn close_connection(&self) {
    //     let mut connection = self.cnn.borrow_mut();
    //     connection.close();
    // }
}