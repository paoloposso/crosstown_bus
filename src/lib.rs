//! # Crosstown Bus
//!
//! `crosstown_bus` is an easy-to-configure bus in Rust with RabbitMQ for event-driven systems.
pub mod tools;
mod bus;
mod message_handler;
mod common;

use std::{error::Error, cell::RefCell};

use amiquip::Connection;
use bus::{Publisher, Subscriber};
pub use common::QueueProperties;
pub use message_handler::MessageHandler;
pub use message_handler::HandleError;

pub struct CrosstownBus();

impl CrosstownBus {
    pub fn new_subscriber(url: String) -> Result<Subscriber, Box<dyn Error>> {
        Ok(Subscriber {
            cnn: RefCell::new(Connection::insecure_open(&url)?)
        })
    }
    
    pub fn new_publisher(url: String) -> Result<Publisher, Box<dyn Error>> {
        Ok(Publisher {
            cnn: RefCell::new(Connection::insecure_open(&url)?)
        })
    }
}