//! # Crosstown Bus
//!
//! `crosstown_bus` is an easy-to-configure bus in Rust with RabbitMQ for event-driven systems.
pub mod tools;
mod queue_bus;
mod event_message;

pub use queue_bus::QueueBus;
pub use event_message::EventMessage;
pub use event_message::MessageHandler;