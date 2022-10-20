//! # Crosstown Bus
//!
//! `crosstown_bus` is an easy-to-configure bus in Rust with RabbitMQ for event-driven systems.

pub mod tools;
mod publisher;
mod subscriber;

pub use publisher::Publisher;
pub use subscriber::Subscriber;