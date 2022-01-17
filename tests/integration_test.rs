#[cfg(test)]
mod integration {
    use event_bus::subscriber::{RabbitSubscriber, Subscriber};

    #[test]
    fn create_rabbit_test() {
        if let Ok(_subs) = RabbitSubscriber::new("amqp://guest:guest@localhost:5672".to_string()) {
            assert!(true);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn create_subscription() {
        if let Ok(subs) = RabbitSubscriber::new("amqp://guest:guest@localhost:5672".to_string()) {
            let _ = subs.subscribe("test".to_string(), |message| {
                println!("{}", message);
                Ok(())
            });
        } else {
            assert!(false);
        }
    }
}
