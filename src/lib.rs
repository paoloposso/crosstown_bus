//! # Crosstown Bus
//!
//! `crosstown_bus` is an easy-to-configure bus in Rust with RabbitMQ for event-driven systems.
pub mod tools;
mod queue_bus;
mod message_handler;
mod common;

use std::{error::Error, cell::RefCell};

use amiquip::Connection;
use queue_bus::{Sender, Receiver};
pub use common::QueueProperties;
pub use message_handler::MessageHandler;
pub use message_handler::HandleError;

pub struct CrosstownBus();

impl CrosstownBus {
    pub fn new_receiver(url: String) -> Result<Receiver, Box<dyn Error>> {
        Ok(Receiver {
            cnn: RefCell::new(Connection::insecure_open(&url)?)
        })
    }
    
    pub fn new_sender(url: String) -> Result<Sender, Box<dyn Error>> {
        Ok(Sender {
            cnn: RefCell::new(Connection::insecure_open(&url)?)
        })
    }
}