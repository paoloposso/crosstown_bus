//! # Crosstown Bus
//!
//! `crosstown_bus` is an easy-to-configure bus in Rust with RabbitMQ for event-driven systems.

use std::{error::Error, thread};

use amiquip::{Connection, QueueDeclareOptions, ConsumerMessage, ConsumerOptions, FieldTable, ExchangeDeclareOptions, Publish};

type HandleResult = Result<(), Box<dyn Error>>;
type SubscribeResult = Result<(), Box<dyn Error>>;
type PublishResult = Result<(), Box<dyn Error>>;

pub struct Bus {
    url: String
}

impl Bus {
    /// Creates a Bus object using Rabbit as Event Broker.
    ///
    /// # Examples
    ///
    /// ```
    /// use crosstown_bus::Bus;
    /// let bus = Bus::new_rabbit_bus("amqp://guest:guest@localhost:5672".to_string()).unwrap();
    /// ```
    pub fn new_rabbit_bus(url: String) -> Result<Bus, Box<dyn Error>> {
        Ok(Bus { url })
    }

    /// Publishes an event.
    ///
    /// # Examples
    ///
    /// ```
    /// pub struct UserUpdated {}
    /// pub struct UserCreated {}

    /// let bus = Bus::new_rabbit_bus("amqp://guest:guest@localhost:5672".to_string()).unwrap();

    /// let _ = bus.subscribe_event::<UserCreated>(String::from("user_created"), String::from("send_email"), |message| {
    /// println!("User CREATED! e-mail sent now: {}", message);
    ///     (false, Ok(()))
    /// });

    /// let _ = bus.subscribe_event::<UserUpdated>(String::from("user_updated"), String::from("send_email"), |message| {
    ///         println!("User Updated! e-mail sent now: {}", message);
    ///     (false, Ok(()))
    /// });

    /// let _ = bus.subscribe_event::<UserUpdated>(String::from("user_updated"), String::from("update_database"), |message| {
    ///     println!("User Updated! Database Updated now: {}", message);
    ///     (false, Ok(()))
    /// });

    /// let res = bus.publish_event::<UserCreated>(String::from("user_created"), String::from("Paolo"));
    /// let _ = bus.publish_event::<UserUpdated>(String::from("user_updated"), String::from("Paolo Victor"));
    /// let _ = bus.publish_event::<UserCreated>(String::from("user_created"), String::from("Thayna"));
    /// ```
    pub fn publish_event<T>(&self, event_name: String, message: String) -> PublishResult {
        let url = self.url.to_owned();
        let event_name = get_event_name::<T>();

        if let Ok(channel) = Connection::insecure_open(&url)?.open_channel(None) {
            let publish_result = channel.basic_publish::<String>(event_name.to_owned(), Publish {
                body: message.as_bytes(),
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

    pub fn subscribe_event<T>(&self, event_name: String, action_name: String, handler: fn(message: String) -> (bool, HandleResult)) -> SubscribeResult {
        let url = self.url.to_owned();

        let event_name = get_event_name::<T>();

        let mut queue_name = event_name.to_owned();
        queue_name.push_str(&String::from("."));
        queue_name.push_str(&action_name);

        thread::spawn(move || {
            match Connection::insecure_open(&url) {
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
            
                                let _ = queue.bind(
                                    &channel.exchange_declare::<String>(amiquip::ExchangeType::Fanout, event_name.to_owned(), exchange_declare_options).unwrap(), 
                                    "".to_string(), FieldTable::new());
            
                                if let Ok(consumer) = queue.consume(ConsumerOptions::default()) {
                                    for message in consumer.receiver().iter() {
                                        match message {
                                            ConsumerMessage::Delivery(delivery) => {
                                                let body = String::from_utf8_lossy(&delivery.body);
                
                                                let handle_result = handler(body.to_string());

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

fn get_event_name<T>() -> String {
    let full_event_name = std::any::type_name::<T>().to_string();
    let event_array = full_event_name.split("::").collect::<Vec<&str>>();
    let event_name = event_array.last().unwrap().to_string();
    event_name
}

// #[cfg(test)]
// mod tests {
//     use crate::Bus;

//     #[test]
//     fn new_bus_works() {
//         let bus = Bus::new_rabbit_bus("url".to_string()).unwrap();
//         if bus.url.len() > 0 {
//             assert!(true);
//         } else {
//             assert!(false);
//         }
//     }
// }
