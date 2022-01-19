#[cfg(test)]
mod integration {
    use event_bus::subscriber::RabbitSubscriber;

    #[test]
    fn create_subscription() {
        if let Ok(subs) = RabbitSubscriber::create_subscription("amqp://guest:guest@localhost:5672".to_string()) {
            let res = subs("test".to_string(), |message| {
                println!("{}", message);
                Ok(())
            });

            match res {
                Ok(_) => assert!(true),
                Err(err) => {
                    println!("{:?}", err);
                    assert!(false);
                },
            }
        } else {
            assert!(false);
        }
    }
}
