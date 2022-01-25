use std::{error::Error, thread};

use amiquip::{Connection, QueueDeclareOptions, ConsumerMessage, ConsumerOptions, FieldTable, AmqpValue, ExchangeDeclareOptions};


type HandleResult = Result<(), Box<dyn Error>>;
type SubscribeResult = Result<(), Box<dyn Error>>;

pub struct RabbitBus {
    url: String
}

impl RabbitBus {
    pub fn new(url: String) -> RabbitBus {
        RabbitBus { url }
    }

    pub fn subscribe_event<'a>(&'a self, event_name: String, handler: fn(message: String) -> HandleResult) -> SubscribeResult {
        let url = self.url.to_owned();
        let queue_name = event_name;

        thread::spawn(move || {
            match Connection::insecure_open(&url) {
                Ok(mut cnn) => {
                    let channel = cnn.open_channel(None).unwrap();
                    
                    let queue = channel.queue_declare(queue_name.to_owned(), QueueDeclareOptions {
                        durable: false,
                        exclusive: false,
                        auto_delete: false,
                        ..Default::default()
                    }).unwrap();

                    let exchange_declare_options = ExchangeDeclareOptions {
                        auto_delete: false,
                        durable: false,
                        internal: false,
                        ..Default::default()
                    };

                    let _ = queue.bind(&channel.exchange_declare::<String>(amiquip::ExchangeType::Fanout, queue_name.to_owned(), exchange_declare_options).unwrap(), 
                        "".to_string(), FieldTable::new());

                    let consumer =  queue.consume(ConsumerOptions::default()).unwrap();
                    
                    for message in consumer.receiver().iter() {
                        match message {
                            ConsumerMessage::Delivery(delivery) => {
                                let body = String::from_utf8_lossy(&delivery.body);

                                let _ = match handler(body.to_string()) {
                                    Ok(_) => { 
                                        let _ = delivery.ack(&channel);
                                    },
                                    Err(error) => { 
                                        println!("Error trying to process message: {:?}", error.as_ref());
                                        let _ = delivery.nack(&channel, false);
                                    },
                                };
                            }
                            other => {
                                println!("Consumer ended: {:?}", other);
                                break;
                            }
                        }
                    }
                },
                Err(err) => println!("[bus] Error trying to create connection: {:?}", err),
            }
        });

        Ok(())
    }
}
