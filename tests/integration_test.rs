
#[cfg(test)]
mod integration {

    use core::time;
    use std::thread;
    use borsh::{BorshDeserialize, BorshSerialize};

    use crosstown_bus::{Bus};
    
    #[test]
    fn create_subscription() {
        #[derive(BorshSerialize, BorshDeserialize, Debug)]
        pub struct UserUpdated {
            name: String,
            id: String
        }

        #[derive(BorshSerialize, BorshDeserialize, Debug)]
        pub struct UserCreated {
            name: String,
            id: String
        }

        let bus = Bus::new_rabbit_bus("amqp://guest:guest@localhost:5672".to_string());

        let _ = bus.subscribe_event::<UserCreated>(String::from("send_email"), |event| {
            println!("E-mail USER CREATED sent TO {}: {:?}", event.name, event);
            (false, Ok(()))
        });

        let _ = bus.subscribe_event::<UserUpdated>(String::from("send_email"), |event| {
            println!("E-mail USER UPDATED sent: {:?}", event);
            (false, Ok(()))
        });

        let _ = bus.subscribe_event::<UserUpdated>(String::from("update_database"), |event| {
            println!("User Updated! Database Updated with user {}: {:?}", event.name, event);
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
            id: "123456".to_owned()
        });

        assert!(res.is_ok());

        let _ = thread::sleep(time::Duration::from_secs(10));
    }
}
