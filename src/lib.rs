//! # Crosstown Bus
//!
//! `crosstown_bus` is an easy-to-configure bus in Rust with RabbitMQ for event-driven systems.
pub mod tools;
mod queue_bus;
mod message_handler;
mod broadcast_publisher;
mod broadcast_subscriber;
mod common;

use std::cell::Cell;
use std::collections::HashMap;

use std::{error::Error, cell::RefCell};

use amiquip::Connection;
use broadcast_subscriber::{SubscriptionManager, BroadcastSubscriber};
use queue_bus::{QueuePublisher, QueueListener};
pub use common::QueueProperties;
pub use message_handler::MessageHandler;
pub use message_handler::HandleError;
pub use broadcast_publisher::BroadcastPublisher;

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

    pub fn new_broadcast_publisher(url: String) -> Result<BroadcastPublisher, Box<dyn Error>> {
        Ok(BroadcastPublisher(
            RefCell::new(Connection::insecure_open(&url)?)
        ))
    }

    pub fn new_broadcast_subscriber<T>(url: String) -> Result<BroadcastSubscriber::<T>, Box<dyn Error>> 
        where T: borsh::BorshSerialize, T: borsh::BorshDeserialize {
        Ok(BroadcastSubscriber {
            cnn: Cell::new(Connection::insecure_open(&url)?),
            subs_manager: SubscriptionManager::<T> { handlers_map: HashMap::default() }
        })
    }
}