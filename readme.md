# Rust Event Bus
Flexible and easy to configure Event Bus for event-driven systems in Rust.

# Supports
- RabbitMQ

# Examples

## Creating a Bus object using Rabbit as the Event Broker

## Subscribing to an Event

First, let's create a subscriber object

```
let subscriber = CrosstownBus::new_queue_subscriber("amqp://guest:guest@localhost:5672".to_owned())?;
```

After that, call the subscribe_event method, passing the event name / queue name that you want to subbscribe to.
If the queue was not created on RabbitMQ, it will be created now, when you subscribe to it.

```
subscriber.subscribe_event("user_created".to_owned(), UserCreatedEventHandler).await;
```
Note that the _subscribe_event_ method in async, therefore, I'm calling _await_ when invoking it.
Another option is to block it, by using the following notation:
```
futures::executor::block_on(subscriber.subscribe_event("user_created".to_owned(), UserCreatedEventHandler));
```

## Publishing an Event
Tp create the publisher the process is pretty much the same, only a different creation method.

```
let mut publisher = CrosstownBus::new_queue_publisher("amqp://guest:guest@localhost:5672".to_owned())?;
```