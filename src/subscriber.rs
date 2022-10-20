use amiquip::{ConsumerOptions,ConsumerMessage};
use amiquip::FieldTable;
use amiquip::ExchangeDeclareOptions;
use amiquip::QueueDeclareOptions;
use amiquip::Connection;
use borsh::BorshDeserialize;
use std::error::Error;
use std::{thread};

use crate::tools::helpers::get_event_name;

pub struct Subscriber {
    url: String,
    // cnn: Box::<Connection>
}

impl Subscriber {
    pub fn new(url: String) -> Self {
        Self { 
            url
        }
    }

    pub fn subscribe_event<T: 'static>(
        &self, 
        action_name: String,
        handler: fn(T) -> (bool, Result<(), Box<dyn Error>>)
    ) -> Result<(), Box<dyn Error>> where T : BorshDeserialize {

        let event_name = get_event_name::<T>();

        let mut queue_name = event_name.to_owned();
        queue_name.push_str(&String::from("."));
        queue_name.push_str(&action_name);

        let url = self.url.to_owned();

        thread::spawn(move || {
            let connection_res = Connection::insecure_open(&url);
            match connection_res {
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

                                if let Ok(exchange) = &channel.exchange_declare::<String>(amiquip::ExchangeType::Fanout, event_name.to_owned(), exchange_declare_options) {
                                    let _ = queue.bind(
                                        exchange, 
                                        "".to_string(), FieldTable::new());
                                        
                                    if let Ok(consumer) = queue.consume(ConsumerOptions::default()) {
                                        for message in consumer.receiver().iter() {
                                            match message {
                                                ConsumerMessage::Delivery(delivery) => {
                                                    let str_message = String::from_utf8_lossy(&delivery.body).to_string();
                                                    let mut buf = str_message.as_bytes();
    
                                                    if let Ok(model) = BorshDeserialize::deserialize(&mut buf) {
                                                        let handle_result = handler(model);
    
                                                        let retry_on_error = handle_result.0;
                                                        let result = handle_result.1;
        
                                                        if result.is_ok() {
                                                            let _ = delivery.ack(&channel);
                                                        } else {
                                                            if retry_on_error {
                                                                let _ = delivery.nack(&channel, true);
                                                            } else {
                                                                let _ = delivery.reject(&channel, false);
                                                            }
                                                        }
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
                                    } else {
                                        eprintln!("[bus] Error trying to consume");
                                    }
                                } else {
                                    eprintln!("[bus] Error declaring exchange");
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