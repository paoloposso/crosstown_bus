#[cfg(test)]
mod integration {
    use event_bus::subscriber::{RabbitSubscriber, Subscriber};

    #[test]
    fn create_rabbit_test() {
        if let Ok(_subs) = RabbitSubscriber::new() {
            assert!(true);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn create_subscription() {
        if let Ok(subs) = RabbitSubscriber::new() {
            let _ = subs.subscribe("test".to_string(), |message| {
                println!("{}", message);
                Ok(())
            });
        } else {
            assert!(false);
        }
    }
}
