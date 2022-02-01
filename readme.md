# Rust Event Bus
Flexible and easy to configure Event Bus for event-driven systems in Rust.

# Supports
- RabbitMQ
- Redis (coming soon)

# Examples

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
- event_name: name of the event to be subscribed to
- action_name: action that will be executed when receiving the message of that event
- handler: function (action) that will be executed when a message is received

```
use crosstown_bus::Bus;

let bus = Bus::new_rabbit_bus("amqp://guest:guest@localhost:5672".to_string()).unwrap();

let _ = bus.subscribe_event(String::from("user_updated"), String::from("update_external_database"), |message| {
    println!("Database Updated now: {}", message);
    (false, Ok(()))
});
```

In this example we are subscribing for the event _user_updated_. The action that will be taken is _update_external_database_. And we also pass a parameter the function that will handle the received message, simulating the sending of an e-mail for notifying the user.

## Publishing an Event

When publishing an event, the publisher, in theory, doesn't know who will handle it.

Then it sends the message as specifies which event it is triggering.

On RabbitMQ this will be represented as the Exchange. See https://www.rabbitmq.com/tutorials/amqp-concepts.html#:~:text=A%20fanout%20exchange%20routes%20messages,the%20broadcast%20routing%20of%20messages.

Crosstown Bus will, if it doesn't exist, create a _fanout exchange_ using the event_name as name for it. This exchange will send a copy of the message for every interested parts, the queues, instead of sending directly to a listener.

This allows an event to be handled by multiple services / microservices as all the subscribers will receive a copy of the message to be handled.

Then, the message sent to the exchange will be broadcasted to all subscribers, allowing all of them to receive and handle the messages, executing the actions independently.

```
use crosstown_bus::Bus;

let bus = Bus::new_rabbit_bus("amqp://guest:guest@localhost:5672".to_string()).unwrap();

let _ = bus.subscribe_event(String::from("user_created"), String::from("send_email"), |message| {
    println!("User CREATED email sent now: {}", message);
    (false, Ok(()))
});

let _ = bus.subscribe_event(String::from("user_updated"), String::from("send_email"), |message| {
    println!("User updated email sent now: {}", message);
    (false, Ok(()))
});

let _ = bus.subscribe_event(String::from("user_updated"), String::from("update_external_database"), |message| {
    println!("Database Updated now: {}", message);
    (false, Ok(()))
});

let _ = bus.publish_event(String::from("user_created"), String::from("Paolo"));
let _ = bus.publish_event(String::from("user_updated"), String::from("Paolo Victor"));
let _ = bus.publish_event(String::from("user_created"), String::from("Thayna"));
```