use std::{cell::RefCell, error::Error, sync::{Mutex, Arc}, thread, collections::BTreeMap};

use amiquip::{Connection, Publish, ConsumerOptions, ConsumerMessage, QueueDeclareOptions, ExchangeDeclareOptions};
use borsh::{BorshSerialize, BorshDeserialize};

use crate::MessageHandler;

pub type GenericResult = Result<(), Box<dyn Error>>;

#[derive(Default)]
pub struct QueueProperties {
    /// queue will be deleted from RabbitMQ when the last consumer disconnects
    pub auto_delete: bool,
    /// if set to true, queue will survive when server restarts
    pub durable: bool,
    pub use_dead_letter: bool
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
        queue_properties: QueueProperties) -> GenericResult
        where T : BorshDeserialize + BorshSerialize + Clone + 'static {

        let connection = Arc::new(Mutex::new(self.cnn));
        let handler = Arc::new(event_handler);
        let queue_name = event_name.clone();
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
                    let exchange_name = get_exchange_name(&queue_name);
                    let exchange = channel.exchange_declare(amiquip::ExchangeType::Direct, exchange_name, ExchangeDeclareOptions::default()).unwrap();

                    _ = queue.bind(&exchange, &queue_name, BTreeMap::default());

                    match queue.consume(ConsumerOptions::default()) {
                        Ok(consumer) => {
                            for message in consumer.receiver().iter() {
                                match message {
                                    ConsumerMessage::Delivery(delivery) => {
                                        let str_message = String::from_utf8_lossy(&delivery.body).to_string();
                                        let mut buf = str_message.as_bytes();
    
                                        if let Ok(model) = BorshDeserialize::deserialize(&mut buf) {
                                            if let Err(err) = handler.handle(model) {
                                                println!("{err}");
                                                _ = delivery.nack(&channel, err.requeue);
                                            } else {
                                                _ = delivery.ack(&channel);
                                            }
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

fn create_dead_letter_policy(queue_name: String, channel: &amiquip::Channel) -> Result<String, Box::<dyn Error>>{
    let dl_ex_name = get_dead_letter_ex_name(&queue_name);
    let dl_exchange = channel.exchange_declare(amiquip::ExchangeType::Fanout, dl_ex_name.to_owned(), ExchangeDeclareOptions::default())?;
    let mut dl_queue_name = queue_name.to_owned();
    dl_queue_name.push_str(&"_dead_letter");
    let dl_queue = channel.queue_declare(dl_queue_name, QueueDeclareOptions { durable: true, exclusive: false, auto_delete: false, arguments: BTreeMap::default() })?;
    _ = dl_queue.bind(&dl_exchange, "".to_owned(), BTreeMap::default());

    Ok(dl_ex_name)
}

fn get_exchange_name(queue_name: &String) -> String {
    let mut exchange_name = queue_name.to_owned();
    exchange_name.push_str("_exchange");
    exchange_name
}

fn get_dead_letter_ex_name(queue_name: &String) -> String {
    let mut exchange_name = queue_name.to_owned();
    exchange_name.push_str("_exchange_dead_letter");
    exchange_name
}

impl QueuePublisher {
    pub fn publish_event<T>(&mut self, event_name: String, message: T)
        -> GenericResult 
            where T: BorshSerialize + BorshDeserialize {

        let exchange_name = get_exchange_name(&event_name);

        let mut buffer = Vec::new();
        message.serialize(&mut buffer)?;
        if let Ok(channel) = self.cnn.get_mut().open_channel(None) {
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