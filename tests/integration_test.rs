
#[cfg(test)]
mod integration {

    use core::time;
    use std::thread;

    use crosstown_bus::{Bus};
    use futures::executor::block_on;

    #[test]
    fn create_subscription() {
        pub struct UserUpdated {}
        pub struct UserCreated {}

        block_on(async {
            let bus = Bus::new_rabbit_bus("amqp://guest:guest@localhost:5672".to_string()).unwrap();

            let _ = bus.subscribe_event::<UserCreated>(String::from("user_created"), String::from("send_email"), |message| {
                println!("User CREATED! e-mail sent now: {}", message);
                (false, Ok(()))
            });

            let _ = bus.subscribe_event::<UserUpdated>(String::from("user_updated"), String::from("send_email"), |message| {
                println!("User Updated! e-mail sent now: {}", message);
                (false, Ok(()))
            });

            let _ = bus.subscribe_event::<UserUpdated>(String::from("user_updated"), String::from("update_database"), |message| {
                println!("User Updated! Database Updated now: {}", message);
                (false, Ok(()))
            });

            let res = bus.publish_event(String::from("user_created"), String::from("Paolo"));
            let _ = bus.publish_event(String::from("user_updated"), String::from("Paolo Victor"));
            let _ = bus.publish_event(String::from("user_created"), String::from("Thayna"));

            assert!(res.is_ok());

            let _ = thread::sleep(time::Duration::from_secs(10));
        });
    }
}
