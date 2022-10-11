//! # Crosstown Bus
//!
//! `crosstown_bus` is an easy-to-configure bus in Rust with RabbitMQ for event-driven systems.

use std::{error::Error, thread};

use amiquip::{Connection, QueueDeclareOptions, ConsumerMessage, ConsumerOptions, FieldTable, ExchangeDeclareOptions, Publish};

use borsh::{BorshDeserialize, BorshSerialize};

type HandleResult = Result<(), Box<dyn Error>>;
type SubscribeResult = Result<(), Box<dyn Error>>;
type PublishResult = Result<(), Box<dyn Error>>;

pub struct Publisher {
    cnn: Box::<Connection>
}

pub struct Subscriber {
    url: String,
    // cnn: Box::<Connection>
}

impl Subscriber {
    pub fn new(url: String) -> Subscriber {
        Subscriber { 
            url
        }
    }


    /// Subscribes to an event. 
    /// The action name represents the action that will be taken after the message is received. It will also be used as queue name.
    /// Multiple queues can be connected to the same exchange (abstracted as Event in this case) and all of them will receive a copy of the message.
    ///
    /// # Example
    /// ```
    /// use crosstown_bus::{Subscriber};
    /// use borsh::{BorshDeserialize, BorshSerialize};
    /// 
    /// #[derive(BorshSerialize, BorshDeserialize, Debug)]
    /// pub struct UserCreated {
    ///     name: String,
    ///     id: String
    /// }

    /// let subs = Subscriber::new("amqp://guest:guest@localhost:5672".to_string());

    /// let _ = subs.subscribe_event::<UserCreated>(String::from("send_email"), |event| {
    ///     println!("E-mail USER CREATED sent TO {}: {:?}", event.name, event);
    ///     (false, Ok(()))
    /// });
    /// ```
    pub fn subscribe_event<T: 'static>(
        &self, 
        action_name: String,
        handler: fn(T) -> (bool, HandleResult)
    ) -> SubscribeResult where T : BorshDeserialize {

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

impl Publisher {
    pub fn new(url: String) -> Result<Publisher, Box<dyn Error>> {
        let cnn = Connection::insecure_open(&url).unwrap();
        Ok(Publisher { 
            cnn: Box::new(cnn)
        })
    }
    
    /// Publishes an event.
    ///
    /// # Example
    /// ```
    /// use crosstown_bus::{Publisher};
    /// use borsh::{BorshDeserialize, BorshSerialize};
    /// 
    /// #[derive(BorshSerialize, BorshDeserialize, Debug)]
    /// pub struct UserCreated {
    ///     name: String,
    ///     id: String
    /// }
    /// 
    /// let mut bus = Publisher::new("amqp://guest:guest@localhost:5672".to_string()).unwrap();
    /// let res = bus.publish_event::<UserCreated>(UserCreated {
    ///     name: "Paolo".to_owned(),
    ///     id: "F458asYfj".to_owned()
    /// });
    /// ```
    pub fn publish_event<T>(&mut self, message: T) -> PublishResult 
        where T: BorshDeserialize + BorshSerialize {
        let event_name = get_event_name::<T>();

        let mut buffer = Vec::new();

        message.serialize(&mut buffer)?;

        if let Ok(channel) = self.cnn.open_channel(None) {
            let publish_result = channel.basic_publish::<String>(event_name.to_owned(), Publish {
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

    /// Closes the Publisher connection to the event broker (RabbitMQ). 
    ///
    /// # Example
    /// ```
    /// use crosstown_bus::{Publisher};
    /// use borsh::{BorshDeserialize, BorshSerialize};
    /// 
    /// #[derive(BorshSerialize, BorshDeserialize, Debug)]
    /// pub struct UserCreated {
    ///     name: String,
    ///     id: String
    /// }
    /// 
    /// let mut publisher = Publisher::new("amqp://guest:guest@localhost:5672".to_string()).unwrap();
    /// let res = publisher.publish_event::<UserCreated>(UserCreated {
    ///     name: "Paolo".to_owned(),
    ///     id: "F458asYfj".to_owned()
    /// });
    /// _ = publisher.disconnect_publisher();
    /// ```
    pub fn disconnect_publisher(self) -> Result<(), Box::<dyn Error>> {
        self.cnn.close()?;
        Ok(())
    }
}

fn get_event_name<T>() -> String {
    let full_event_name = std::any::type_name::<T>().to_string();
    let event_array = full_event_name.split("::").collect::<Vec<&str>>();
    event_array.last().unwrap().to_string()
}