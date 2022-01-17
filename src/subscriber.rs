use std::{error::Error};
use std::{thread, time};

type ConsumeResult = Result<(), Box<dyn Error>>;
type SubscribeResult = Result<Box<dyn Subscriber>, Box<dyn Error>>;

pub trait Subscriber {
    fn subscribe(&self, event_name: String, handler: fn(message: String) -> ConsumeResult) -> SubscribeResult;
}

pub struct RabbitSubscriber;

impl RabbitSubscriber {
    pub fn new() -> Result<RabbitSubscriber, Box<dyn Error>> {
        Ok(RabbitSubscriber)
    }
}

impl Subscriber for RabbitSubscriber {
    fn subscribe(&self, event_name: String, handler: fn(message: String) -> ConsumeResult) -> SubscribeResult {
        loop {
            let ten_millis = time::Duration::from_millis(2500);
            let now = time::Instant::now();
            thread::sleep(ten_millis);
            let _ = handler(ToString::to_string(&"test"));
        }
    }
}
