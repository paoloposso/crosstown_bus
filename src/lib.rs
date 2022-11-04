//! # Crosstown Bus
//!
//! `crosstown_bus` is an easy-to-configure bus in Rust with RabbitMQ for event-driven systems.
pub mod tools;
mod queue_subscriber;
mod queue_publisher;
mod event_message;

pub use queue_subscriber::QueueSubscriber;
pub use queue_publisher::QueuePublisher;
pub use event_message::MessageHandler;