use std::{error::Error, pin::Pin, thread, borrow::Borrow};

use amiquip::{Connection, Result, QueueDeclareOptions, ConsumerMessage, ConsumerOptions};
use futures::Future;

type HandleResult = Result<(), Box<dyn Error>>;

pub struct RabbitBus {
    url: String,
    queue: String
}

impl RabbitBus {

    pub fn new(url: String, queue: String) -> RabbitBus {
        RabbitBus { url, queue }
    }

    pub async fn subscribe(&self, handler: fn(message: String) -> HandleResult) -> HandleResult {
        if let Ok(mut cnn) = Connection::insecure_open(&self.url) {
            if let Ok(channel) = cnn.open_channel(None) {
                if let Ok(queue) = channel.queue_declare(&self.queue, QueueDeclareOptions::default()) {
                    if let Ok(consumer) = queue.consume(ConsumerOptions::default()) {
                        async move {
                            for message in consumer.receiver().iter() {
                                match message {
                                    ConsumerMessage::Delivery(delivery) => {
                                        let body = String::from_utf8_lossy(&delivery.body);
                                        if let Ok(_) = handler(body.to_string()) {
                                            consumer.ack(delivery).unwrap();  
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
                        }.await;
                    }
                }
            }
        }
        Ok(())
    }
}
