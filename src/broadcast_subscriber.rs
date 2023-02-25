use amiquip::{ConsumerOptions, ConsumerMessage, QueueDeclareOptions, 
    Connection, Queue};
use borsh::{BorshDeserialize, BorshSerialize};
use std::borrow::{Borrow, BorrowMut};
use std::cell::Cell;
use std::collections::{HashMap, BTreeMap};
use std::error::Error;
use std::fmt::format;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::{MessageHandler, QueueProperties};
use crate::tools::helpers::{create_exchange};

pub type GenericResult = Result<(), Box<dyn Error>>;

pub struct SubscriptionManager<T> {
    pub handlers_map: HashMap<Arc<String>, Vec<Arc<dyn MessageHandler<T> + Send + Sync>>>,
}

pub struct BroadcastSubscriber<T> where T : BorshSerialize + BorshDeserialize {
    pub(crate) cnn: Cell<Connection>,
    pub(crate) subs_manager: SubscriptionManager<T>
}

impl<T> BroadcastSubscriber<T> where T : BorshSerialize + BorshDeserialize + Clone + 'static {
    pub fn new(url: String) -> Result<Self, Box<dyn Error>> {
        Ok(BroadcastSubscriber::<T> {
            cnn: Cell::new(Connection::insecure_open(&url)?),
            subs_manager: SubscriptionManager::<T> { handlers_map: HashMap::new() }
        })
    }

    pub fn add_subscription(&mut self, event_name: String, 
        handler: Arc<dyn MessageHandler<T> + Send + Sync>
    ) -> Result<&Self, Box<dyn Error>> {        
        if let Some(list) = self.subs_manager.handlers_map.get_mut(&event_name) {
            list.push(handler);
        } else {
            let mut handlers_list = Vec::new();
            handlers_list.push(handler);
            self.subs_manager.handlers_map.insert(Arc::new(event_name), handlers_list.clone());
        }
        Ok(self)
    }

    pub fn subscribe_registered_events(self, queue_properties: QueueProperties) -> GenericResult {
        let handlers = self.subs_manager.handlers_map;
        let connection = Arc::new(Mutex::new(self.cnn));
        
        for (event_name, handlers_list) in handlers {
            for handler in handlers_list  {
                let cnn = Arc::clone(&connection);
                let event_arc =  Arc::clone(&event_name);

                thread::spawn(move || {
                    let event = event_arc.borrow();
                    let channel = cnn.lock().unwrap().get_mut().open_channel(None).unwrap();
                    let queue: Queue = channel.queue_declare(event, QueueDeclareOptions {
                        durable: queue_properties.durable,
                        exclusive: false,
                        auto_delete: queue_properties.auto_delete,
                        ..Default::default()
                    }).unwrap();
                    let exchange = create_exchange(event, "fanout".to_owned(), &channel);
                    _ = queue.bind(&exchange, event.to_owned(), BTreeMap::default());
                    match queue.borrow().consume(ConsumerOptions::default()) {
                        Ok(consumer) => {
                            for message in consumer.receiver().iter() {
                                match message {
                                    ConsumerMessage::Delivery(delivery) => {
                                        send_to_handler(delivery, &handler, &channel);
                                    }
                                    other => {
                                        println!("Consumer ended: {:?}", other);
                                        break;
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            eprintln!("[bus] Error trying to consume messages");
                        }
                    };
                });
            }
        }
        Ok(())
    }

    pub fn close_connection(self) -> GenericResult {
        self.cnn.into_inner().close()?;
        Ok(())
    }
}

fn send_to_handler<T>(delivery: amiquip::Delivery, handler: &Arc<dyn MessageHandler<T> + Send + Sync>, channel: &amiquip::Channel)
    where T : BorshDeserialize + BorshSerialize + Clone + 'static {
    let str_message = String::from_utf8_lossy(&delivery.body).to_string();
    let mut buf = str_message.as_bytes();
    if let Ok(model) = BorshDeserialize::deserialize(&mut buf) {
        _ = handler.handle(model);
        _ = delivery.ack(channel);
    } else {
        _ = delivery.nack(channel, false);
        eprintln!("[bus] Error trying to desserialize. Check message format. Message: {:?}", str_message);
    }
}
