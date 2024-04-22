use std::{cell::RefCell, sync::{Mutex, Arc}, thread, collections::BTreeMap};

use amiquip::{Connection, Publish, ConsumerOptions, ConsumerMessage, QueueDeclareOptions};
use borsh::{BorshSerialize, BorshDeserialize};

use crate::{MessageHandler, tools::helpers::{create_dead_letter_policy, create_exchange, GenericResult}, message_handler::send_message_to_handler, QueueProperties};

pub struct Publisher {
    pub(crate) cnn: RefCell<Connection>
}

pub struct Subscriber {
    pub(crate) cnn: RefCell<Connection>
}

impl Subscriber {
    pub fn subscribe<T>(self, event_name: String, 
        event_handler: impl MessageHandler<T> + Send + Sync + 'static,
        queue_properties: QueueProperties) -> GenericResult
        where T : BorshDeserialize + BorshSerialize + Clone + 'static {

        let connection = Arc::new(Mutex::new(self.cnn));
        let handler = Arc::new(event_handler);
        let mut queue_name = event_name.clone();
        if queue_properties.consume_queue_name.is_some() {
            queue_name = queue_properties.consume_queue_name.unwrap();
        }
        let cnn = Arc::clone(&connection);

        thread::spawn(move || {
            let channel = cnn.lock().unwrap().get_mut().open_channel(None).unwrap();

            let mut queue_options = QueueDeclareOptions::default();
            queue_options.auto_delete = queue_properties.auto_delete;
            queue_options.durable = queue_properties.durable;

            if queue_properties.use_dead_letter {
                let dl_ex_name = create_dead_letter_policy(queue_name.to_owned(), &channel).unwrap();
                queue_options.arguments.insert("x-dead-letter-exchange".to_owned(), amiquip::AmqpValue::LongString(dl_ex_name));
            }

            match channel.queue_declare(&queue_name, queue_options) {
                Ok(queue) => {
                    let ex_name = crate::tools::helpers::get_exchange_name(&event_name);
                    let exchange = create_exchange(&ex_name, "fanout".to_owned(), &channel);

                    _ = queue.bind(&exchange, &queue_name, BTreeMap::default());

                    match queue.consume(ConsumerOptions::default()) {
                        Ok(consumer) => {
                            for message in consumer.receiver().iter() {
                                match message {
                                    ConsumerMessage::Delivery(delivery) => {
                                        send_message_to_handler(delivery, &handler, &channel);
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

impl Publisher {
    pub fn send<T>(&mut self, event_name: String, message: T) -> GenericResult 
                                            where T: BorshSerialize + BorshDeserialize {
        let mut buffer = Vec::new();
        message.serialize(&mut buffer)?;
        if let Ok(channel) = self.cnn.get_mut().open_channel(None) {
            let exchange_name = crate::tools::helpers::get_exchange_name(&event_name);
            let _ = create_exchange(&exchange_name, "fanout".to_owned(), &channel);
            let publish_result = channel.basic_publish::<String>(
                exchange_name,
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