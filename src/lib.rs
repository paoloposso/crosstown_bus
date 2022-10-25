//! # Crosstown Bus
//!
//! `crosstown_bus` is an easy-to-configure bus in Rust with RabbitMQ for event-driven systems.

pub mod tools;
mod bus;
mod message;

pub use bus::Bus;
pub use message::Message;
pub use message::MessageHandler;