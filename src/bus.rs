use amiquip::{ConsumerOptions, ConsumerMessage, QueueDeclareOptions, 
    Connection, Publish};
use borsh::{BorshDeserialize, BorshSerialize};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;
use crate::Message;
use crate::message::MessageHandler;
use crate::tools::helpers::get_event_name;

pub struct Bus {
    cnn: Box::<Connection>,
    subs_manager: SubscriptionManager
}

pub struct SubscriptionManager {
    pub handlers: HashMap<String, Vec<Rc<dyn MessageHandler<String>>>>
}

impl Bus {
    pub fn new(url: String) -> Self {
        Self { 
            cnn: Box::new(Connection::insecure_open(&url).unwrap()),
            subs_manager: SubscriptionManager { handlers: HashMap::new() }
        }
    }

    pub fn add_subscription<T>(&mut self, event_name: String, handler: Rc<dyn MessageHandler<String>>) 
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

    pub fn publish_event<T>(&mut self, event_name: String, message: Message::<T>) -> Result<(), Box<dyn Error>> 
        where T: BorshDeserialize + BorshSerialize {
        // let event_name = get_event_name::<T>();

        let mut buffer = Vec::new();

        message.serialize(&mut buffer)?;

        if let Ok(channel) = self.cnn.open_channel(None) {
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

    pub async fn subscribe_registered_events(mut self) {

        for (event_name, subscriptions) in self.subs_manager.handlers.iter_mut() {
            for subs in subscriptions {
                let queue_name = event_name.to_owned();
                let channel = self.cnn.open_channel(None).unwrap();
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
            }
        }
    }
}