//! # Crosstown Bus
//!
//! `crosstown_bus` is an easy-to-configure bus in Rust with RabbitMQ for event-driven systems.
pub mod tools;
mod queue_bus;
mod message_handler;
mod broadcast_publisher;
mod broadcast_subscriber;
mod common;

use std::{error::Error, cell::RefCell};

use amiquip::Connection;
use queue_bus::{Sender, Receiver};
pub use common::QueueProperties;
pub use message_handler::MessageHandler;
pub use message_handler::HandleError;
pub use broadcast_publisher::BroadcastPublisher;

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

    // pub fn new_broadcast_publisher(url: String) -> Result<BroadcastPublisher, Box<dyn Error>> {
    //     Ok(BroadcastPublisher(
    //         RefCell::new(Connection::insecure_open(&url)?)
    //     ))
    // }

    // pub fn new_broadcast_subscriber<T>(url: String) -> Result<BroadcastSubscriber::<T>, Box<dyn Error>> 
    //     where T: borsh::BorshSerialize, T: borsh::BorshDeserialize {
    //     Ok(BroadcastSubscriber {
    //         cnn: Cell::new(Connection::insecure_open(&url)?),
    //         subs_manager: SubscriptionManager::<T> { handlers_map: HashMap::default() }
    //     })
    // }
}