
#[cfg(test)]
mod integration {

    use core::time;
    use std::thread;

    use crosstown_bus::new_rabbit_bus;
    use futures::executor::block_on;

    #[test]
    fn create_subscription() {
        block_on(async {
            let bus = new_rabbit_bus("amqp://guest:guest@localhost:5672".to_string());

            let _ = bus.subscribe_event(String::from("user_created"), String::from("send_email"), |message| {
                println!("User CREATED email sent now: {}", message);
                (false, Ok(()))
            });

            let _ = bus.subscribe_event(String::from("user_updated"), String::from("send_email"), |message| {
                println!("User updated email sent now: {}", message);
                (false, Ok(()))
            });

            let _ = bus.subscribe_event(String::from("user_updated"), String::from("update_database"), |message| {
                println!("Database Updated now: {}", message);
                (false, Ok(()))
            });

            let _ = bus.publish_event(String::from("user_updated"), String::from("user updated 1"));

            let _ = thread::sleep(time::Duration::from_secs(10));
        });
    }
}
