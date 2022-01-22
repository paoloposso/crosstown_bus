
#[cfg(test)]
mod integration {

    use event_bus::subscriber::RabbitBus;
    use futures::{executor::block_on, future::join, join};
    use tokio::select;

    #[test]
    fn create_subscription() {

        block_on(async {

            let bus = RabbitBus::new("amqp://guest:guest@localhost:5672".to_string(), String::from("user_created"));

            let subscribe_async = bus.subscribe(|message| {
                println!("Created now: {}", message);
                Ok(())
            });

            // let subscribe_asyncx = bus.subscribe("user_updated".to_string(), |message| {
            //     println!("Updated: {}", message);
            //     Ok(())
            // });

            let _ = subscribe_async.await;
            // subscribe_asyncx.await;
        });
    }
}
