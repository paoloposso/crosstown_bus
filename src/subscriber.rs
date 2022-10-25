use amiquip::{ConsumerOptions, ConsumerMessage, QueueDeclareOptions, 
    Connection};
use borsh::BorshDeserialize;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;
use crate::message::MessageHandler;

pub struct Subscriber {
    url: String,
    subs_manager: SubscriptionManager
}

pub struct SubscriptionManager {
    handlers: HashMap<String, Vec<Rc<dyn MessageHandler<String>>>>
}

impl Subscriber {
    pub fn new(url: String) -> Self {
        Self { 
            url,
            subs_manager: SubscriptionManager { handlers: HashMap::new() }
        }
    }

    pub fn add_subscription<T>(mut self, event_name: String, handler: Rc<dyn MessageHandler<String>>) 
        -> Result<(), Box<dyn Error>> where T: ?Sized {
        if self.subs_manager.handlers.contains_key(&event_name) {
            self.subs_manager.handlers.get_mut(&event_name).unwrap().push(handler);
        } else {
            let mut handler_list: Vec<Rc<dyn MessageHandler<String>>> = Vec::new();
            handler_list.push(handler);
            self.subs_manager.handlers.insert(event_name, handler_list);
        }
        Ok(())
    }

    pub fn subscribe_registered_events(mut self) {
        let url = self.url.to_owned();

        for (event_name, subscriptions) in self.subs_manager.handlers.iter_mut() {
            for subs in subscriptions {
                let queue_name = event_name.to_owned();
                let mut cnn = Connection::insecure_open(&url).unwrap();
                let channel = cnn.open_channel(None).unwrap();
                let queue = channel.queue_declare(queue_name.to_owned(), QueueDeclareOptions {
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
                                        _ = subs.handle(model);
                                        _ = delivery.ack(&channel);
                                    } else {
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
            }
        }
    }
}