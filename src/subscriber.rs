use std::{error::Error};
use std::sync::mpsc::channel;
use std::thread;


use amiquip::{Connection, Result, Channel, QueueDeclareOptions, ConsumerMessage, ConsumerOptions};

type MessageConsumedResult = Result<(), Box<dyn Error>>;
type CreateSubscriberResult = Result<Box<RabbitSubscriber>, Box<dyn Error>>;

pub trait Subscriber {
    // async fn subscribe(&self, event_name: String, handler: fn(message: String) -> MessageConsumedResult) -> Result<(), Box<dyn Error>>;
}

pub struct RabbitSubscriber {
    channel: Box<Channel>,
}

impl RabbitSubscriber {
    pub fn new(url: String) -> CreateSubscriberResult {        
        Ok(Box::new(RabbitSubscriber {
            channel: Box::new(Connection::insecure_open(&url)?.open_channel(None)?)
        }))
    }
}

impl RabbitSubscriber {
    pub async fn subscribe(&self, event_name: String, handler: fn(message: String) -> MessageConsumedResult) -> Result<(), Box<dyn Error>> {
        
        let queue = self.channel.queue_declare(&event_name, QueueDeclareOptions::default())?;
        let consumer = queue.consume(ConsumerOptions::default())?;

        for message in consumer.receiver().iter() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    let body = String::from_utf8_lossy(&delivery.body);
                    if let Ok(_) = handler(body.to_string()) {
                        consumer.ack(delivery)?;
                    } else {
                        let _ = consumer.nack(delivery, false);
                    }
                }
                other => {
                    println!("Consumer ended: {:?}", other);
                    break;
                }
            }
        }

        Ok(())
    }
}
