
#[cfg(test)]
mod integration {

    use core::time;
    use std::{thread};

    use rusty_bus::{Bus, new_rabbit_bus};
    use futures::executor::block_on;

    #[test]
    fn create_subscription() {
        block_on(async {
            let bus = new_rabbit_bus("amqp://guest:guest@localhost:5672".to_string());

            let _ = bus.subscribe_event(String::from("user_created"), |message| {
                println!("Created now: {}", message);
                //assert!(true);
                Ok(())
            });

            let _ = bus.subscribe_event(String::from("user_updated"), |message| {
                println!("Updated now: {}", message);
                //assert!(true);
                Ok(())
            });

            let _ = bus.publish_event(String::from("user_updated"), String::from("user updated 1"));
            let _ = bus.publish_event(String::from("user_updated"), String::from("user updated 2"));
            let _ = bus.publish_event(String::from("user_created"), String::from("paolo"));

            let _ = thread::sleep(time::Duration::from_secs(10));
            //assert!(false);
        });
    }
}
