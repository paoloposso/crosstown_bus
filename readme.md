# Rust Event Bus
Flexible and easy to configure Event Bus for event-driven systems in Rust.

# Supports
- RabbitMQ

# Examples


## Creating the structs for your project

The structs that will be published by Crosstown Bus must derive from Serialize and Deserialize traits.

```
#[derive(Serialize, Deserialize)]
pub struct UserUpdated {
    name: String,
    id: String
}
```
They can be passed as parameter to the publisher method and deserialized inside of the handler.
When publishing and subscribing events it's necessary to inform the struct type via generics.
See https://doc.rust-lang.org/rust-by-example/generics.html to understand more about generics in Rust.


## Creating a Bus object using Rabbit as the Event Broker
```
use crosstown_bus::Bus;

let bus = Bus::new_rabbit_bus("amqp://guest:guest@localhost:5672".to_string()).unwrap();
```

## Subscribing to an Event

A service / microservice is responsible for handling an event (the subscriber service), sent by another service (the publisher).

Then, with Rabbit, it can subscribe to an event by creating a queue, which listen to an exchange. See https://www.rabbitmq.com/tutorials/amqp-concepts.html#:~:text=A%20fanout%20exchange%20routes%20messages,the%20broadcast%20routing%20of%20messages.

For example: when an user is updated, an external database must be updated.

- When the user is updated, the service that updates the user sends a message to trigger the event _user_updated_;

- The service that must update the external database will subscribe to this event;

- The subscriber (or all subscribers) will receive a copy of the broadcasted message.

We must also inform the action that will be taken, (i.e. Update Database, Send E-mail etc.). This way it's possible to make multiple subscribers to handle the same event independently.

### Parameters:
- action_name: action that will be executed when receiving the message of that event
- handler: function (action) that will be executed when a message is received

```
use crosstown_bus::Bus;

let bus = Bus::new_rabbit_bus("amqp://guest:guest@localhost:5672".to_string()).unwrap();

let _ = bus.subscribe_event::<UserCreated>(String::from("send_email"), |message| {
    println!("User CREATED! e-mail sent now: {}", message);
    (false, Ok(()))
});
```

In this example we are subscribing for the event _user_updated_. The action that will be taken is _update_external_database_. And we also pass a parameter the function that will handle the received message, simulating the sending of an e-mail for notifying the user.

## Publishing an Event

When publishing an event, the publisher, in theory, doesn't know who will handle it.

Then it sends the message as specifies which event it is triggering.

In RabbitMQ this will be represented by the Exchange. See https://www.rabbitmq.com/tutorials/amqp-concepts.html#:~:text=A%20fanout%20exchange%20routes%20messages,the%20broadcast%20routing%20of%20messages.

Crosstown Bus will, if it doesn't exist, create a _fanout exchange_ using the event_name as name for it. This exchange will send a copy of the message for every interested parts, the queues, instead of sending directly to a listener.

This allows an event to be handled by multiple services / microservices as all the subscribers will receive a copy of the message to be handled.

Then, the message sent to the exchange will be broadcasted to all subscribers, allowing all of them to receive and handle the messages, executing the actions independently.

```
use crosstown_bus::Bus;

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
```