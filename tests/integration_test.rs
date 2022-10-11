
#[cfg(test)]
mod integration {

    use core::time;
    use std::thread;
    use borsh::{BorshDeserialize, BorshSerialize};

    use crosstown_bus::{Subscriber, Publisher};
    
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
    
    #[test]
    fn create_subscription() {
        let subscriber = Subscriber::new("amqp://guest:guest@localhost:5672".to_string());
        let mut publisher = Publisher::new("amqp://guest:guest@localhost:5672".to_string()).unwrap();

        let _ = subscriber.subscribe_event::<UserCreated>(String::from("send_email"), |event| {
            println!("E-mail USER CREATED sent TO {}: {:?}", event.name, event);
            (false, Ok(()))
        });

        let _ = subscriber.subscribe_event::<UserUpdated>(String::from("send_email"), |event| {
            println!("E-mail USER UPDATED sent: {:?}", event);
            (false, Ok(()))
        });

        if subscriber.subscribe_event::<UserUpdated>(
            String::from("update_database"), 
            handle_event).is_ok() {
                assert_eq!(true, true);
            }

        let res = publisher.publish_event::<UserCreated>(UserCreated {
            name: "Paolo".to_owned(),
            id: "F458asYfj".to_owned()
        });

        let _ = publisher.publish_event::<UserCreated>(UserCreated {
            name: "Thayna".to_owned(),
            id: "PkjioYHb".to_owned()
        });

        let _ = publisher.publish_event::<UserUpdated>(UserUpdated {
            name: "Thayna T".to_owned(),
            id: "123456".to_owned()
        });

        _ = publisher.disconnect_publisher();

        assert!(res.is_ok());

        let _ = thread::sleep(time::Duration::from_secs(10));
    }

    fn handle_event(event: UserUpdated) -> (bool, Result<(), Box<dyn std::error::Error>>) {
        println!("E-mail USER UPDATED sent: {:?}", event);
        (false, Ok(()))
    }
}
