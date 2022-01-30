# Rust Event Bus
Flexible and easy to configure Event Bus for event-driven systems in Rust.

# Supports
- RabbitMQ

# Examples

## Creating a Bus object using Rabbit as the Event Broker.     
```
let bus = new_rabbit_bus("amqp://guest:guest@localhost:5672".to_string()).unwrap();
pub fn new_rabbit_bus(url: String) -> Result<Bus, Box<dyn Error>> {
    let channel = Connection::insecure_open(&url)?.open_channel(None)?;
    Ok(Bus { url, channel })
}
```

## Publishing an Event
```
let bus = new_rabbit_bus("amqp://guest:guest@localhost:5672".to_string()).unwrap();

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

let _ = bus.publish_event(String::from("user_created"), String::from("Paolo"));
let _ = bus.publish_event(String::from("user_updated"), String::from("Paolo Victor"));
let _ = bus.publish_event(String::from("user_created"), String::from("Thayna"));
```

## Subscribing to an event.
Receives:
- event_name: name of the event to be expected
- action_name: action that will be executed when receiving the message for that event
- handler: closure (action) that will be executed when a message is received

```
let bus = new_rabbit_bus("amqp://guest:guest@localhost:5672".to_string()).unwrap();

let _ = bus.subscribe_event(String::from("user_updated"), String::from("update_database"), |message| {
    println!("Database Updated now: {}", message);
    (false, Ok(()))
});
```