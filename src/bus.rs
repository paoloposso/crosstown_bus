use amiquip::{ConsumerOptions, ConsumerMessage, QueueDeclareOptions, 
    Connection, Publish, Queue};
use borsh::{BorshDeserialize, BorshSerialize};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::EventMessage;
use crate::event_message::MessageHandler;

pub struct Bus {
    pub cnn: Mutex<RefCell<Connection>>,
    subs_manager: SubscriptionManager
}

pub struct SubscriptionManager {
    pub handlers_map: HashMap<String, Vec<Arc<dyn MessageHandler<String> + Send + Sync>>>,
}

impl Bus {
    pub fn new(url: String) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            cnn: Mutex::new(RefCell::new(Connection::insecure_open(&url)?)),
            subs_manager: SubscriptionManager { handlers_map: HashMap::new() }
        })
    }

    pub fn add_subscription<T>(mut self, event_name: String, 
        handler: Arc<dyn MessageHandler<String> + Send + Sync>
    ) -> Result<Self, Box<dyn Error>> where T: ?Sized {
        if let Some(list) = self.subs_manager.handlers_map.get_mut(&event_name) {
            list.push(handler);
        } else {
            let mut handlers_list = Vec::new();
            handlers_list.push(handler);
            self.subs_manager.handlers_map.insert(event_name, handlers_list.clone());
        }
        
        Ok(self)
    }

    pub fn publish_event<T>(&mut self, event_name: String, message: EventMessage::<T>) 
        -> Result<(), Box<dyn Error>> where T: BorshSerialize + BorshDeserialize {
        let mut buffer = Vec::new();
        message.serialize(&mut buffer)?;

        if let Ok(channel) = self.cnn.lock().unwrap().get_mut().open_channel(None) {
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
        let handlers = self.subs_manager.handlers_map;
        let connection = Arc::new(self.cnn);
        let mut tasks = vec![];
        for (event_name, handlers_list) in handlers {
            for handler in handlers_list  {
                let queue_name = event_name.clone();
                let cnn = Arc::clone(&connection);
                tasks.push(thread::spawn(move || {
                    let channel = cnn.lock().unwrap().get_mut().open_channel(None).unwrap();
                    let queue: Queue = channel.queue_declare(queue_name, QueueDeclareOptions {
                        durable: false,
                        exclusive: false,
                        auto_delete: false,
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
                }));
            }
        }
        Ok(())
    }

    pub fn close_connection(self) {
        let mut cnn = self.cnn.lock().unwrap();
        let a = cnn.get_mut();
    }
}