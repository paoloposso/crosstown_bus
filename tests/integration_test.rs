
#[cfg(test)]
mod integration {

    use core::time;
    use std::thread;
    use serde::Serialize;
    use serde::Deserialize;

    use crosstown_bus::{Bus};
    
    #[test]
    fn create_subscription() {
        #[derive(Serialize, Deserialize)]
        pub struct UserUpdated {
            name: String,
            id: String
        }

        #[derive(Serialize, Deserialize)]
        pub struct UserCreated {
            name: String,
            id: String
        }

        let bus = Bus::new_rabbit_bus("amqp://guest:guest@localhost:5672".to_string()).unwrap();

        let _ = bus.subscribe_event::<UserCreated>(String::from("send_email"), |message| {
            println!("User CREATED! e-mail sent now: {}", message);
            (false, Ok(()))
        });

        let _ = bus.subscribe_event::<UserUpdated>(String::from("send_email"), |message| {
            println!("User Updated! e-mail sent now: {}", message);
            (false, Ok(()))
        });

        let _ = bus.subscribe_event::<UserUpdated>(String::from("update_database"), |message| {
            println!("User Updated! Database Updated now: {}", message);
            (false, Ok(()))
        });

        let res = bus.publish_event::<UserCreated>(UserCreated {
            name: "Paolo".to_owned(),
            id: "F458asYfj".to_owned()
        });

        let _ = bus.publish_event::<UserCreated>(UserCreated {
            name: "Thayna".to_owned(),
            id: "PkjioYHb".to_owned()
        });

        let _ = bus.publish_event::<UserUpdated>(UserUpdated {
            name: "Thayna T".to_owned(),
            id: "PkjioYHb".to_owned()
        });

        assert!(res.is_ok());

        let _ = thread::sleep(time::Duration::from_secs(10));
    }
}
