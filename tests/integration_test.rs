
#[cfg(test)]
mod integration {

    use core::time;
    use std::{thread, error::Error};

    use event_bus::bus::RabbitBus;
    use futures::executor::block_on;

    #[test]
    fn create_subscription() {
        block_on(async {
            let bus = RabbitBus::new("amqp://guest:guest@localhost:5672".to_string());

            let _ = bus.subscribe_event(String::from("user_createdx"), |message| {
                println!("Created now: {}", message);
                Ok(())
            });

            let _ = bus.subscribe_event(String::from("user_updatedx"), |message| {
                println!("Updated now: {}", message);
                Ok(())
            });

            let _ = thread::sleep(time::Duration::from_secs(50));
        });
    }
}
