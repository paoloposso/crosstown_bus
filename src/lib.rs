//! # Crosstown Bus
//!
//! `crosstown_bus` is an easy-to-configure bus in Rust with RabbitMQ for event-driven systems.
pub mod tools;
mod queue_bus;
mod message_handler;
// mod broadcast_publisher;
// mod broadcast_subscriber;
mod common;

use std::{error::Error, cell::RefCell};

use amiquip::Connection;
use queue_bus::{QueuePublisher, QueueListener};
pub use common::QueueProperties;
pub use message_handler::MessageHandler;
pub use message_handler::HandleError;
// pub use broadcast_publisher::BroadcastPublisher;
// pub use broadcast_subscriber::BroadcastSubscriber;

pub struct CrosstownBus();

impl CrosstownBus {
    pub fn new_queue_listener(url: String) -> Result<QueueListener, Box<dyn Error>> {
        Ok(QueueListener {
            cnn: RefCell::new(Connection::insecure_open(&url)?)
        })
    }
    
    pub fn new_queue_publisher(url: String) -> Result<QueuePublisher, Box<dyn Error>> {
        Ok(QueuePublisher {
            cnn: RefCell::new(Connection::insecure_open(&url)?)
        })
    }
}