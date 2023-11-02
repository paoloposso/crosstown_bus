#[derive(Default)]
pub struct QueueProperties {
    /// queue will be deleted from RabbitMQ when the last consumer disconnects
    pub auto_delete: bool,
    /// if set to true, queue will survive when server restarts
    pub durable: bool,
    pub use_dead_letter: bool,
    pub consume_queue_name: Option<String>,
}
