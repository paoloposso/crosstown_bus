//! # Crosstown Bus
//!
//! `crosstown_bus` is an easy-to-configure bus in Rust with RabbitMQ for event-driven systems.
pub mod tools;
mod queue_bus;
mod event_message;

use std::{error::Error, cell::RefCell};

use amiquip::Connection;
use queue_bus::{QueuePublisher, QueueSubscriber};
pub use event_message::MessageHandler;

pub struct CrosstownBus();

impl CrosstownBus {
    pub fn new_queue_subscriber(url: String) -> Result<QueueSubscriber, Box<dyn Error>> {
        Ok(QueueSubscriber {
            cnn: RefCell::new(Connection::insecure_open(&url)?)
        })
    }
    pub fn new_queue_publisher(url: String) -> Result<QueuePublisher, Box<dyn Error>> {
        Ok(QueuePublisher {
            cnn: RefCell::new(Connection::insecure_open(&url)?)
        })
    }
}