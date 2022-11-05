# Rust Event Bus
Flexible and easy to configure Event Bus for event-driven systems in Rust.

Create your own structs and integrate your services / microservices by sending strongly typed payloads.

# Supports
- RabbitMQ

# Examples

## Creating a Handler
A message received from the queue will be sent to a Message Handler.

Your Message Handler must implement the _trait_ MessageHandler, and inform the type of Message it you handle.

```
struct UserCreatedEventHandler;

impl MessageHandler<UserCreatedEventMessage> for UserCreatedEventHandler {
    fn handle(&self, message: Box<UserCreatedEventMessage>) -> Result<(), String> {
        println!("Message received on User Created Handler: {:?}", message);
        Ok(())
    }
}
```

## Creating an Event Message
Notice that the Message type we want to send and receive between services is **UserCreatedEventMessage**.

When we create the Event Message struct and the Message Handler, we are defining:
- the format of the event message we are sending between services.
- the struct/method that will handle the incoming messages.

Here you have an example of Message, that we are using in the current example:

```
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct UserCreatedEventMessage {
    pub user_id: String,
    pub user_name: String
}
```
Notice that your struct must derive from BorshDeserialize and BorshSerialize, so Crosstow Bus is able to serialize the struct you defined to send to RabbitMQ and desserialize the messages coming from RabbitMQ into your customized format.

So, don't forget to add the imports to youw cargo.toml file.
```
borsh = "0.9.3"
borsh-derive = "0.9.1"
```

## Subscribing to an Event to receive the Messages
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

## Publishing / sending an Event Message
To create the publisher the process is pretty much the same, only a different creation method.

```
let mut publisher = CrosstownBus::new_queue_publisher("amqp://guest:guest@localhost:5672".to_owned())?;

_ = publisher.publish_event("user_created".to_owned(), 
        UserCreatedEventMessage {
            user_id: "asdf".to_owned(),
            user_name: "Billy Gibbons".to_owned()
        });
```
Since the method publish_event receives a generic parameter as the Message, we can use the same publisher object to publish multiple objects types to multiple queues.
**Warning:** if the message type you are publishing on a queue doesn't match what the subscriber handler is expecting, it will not be possible to parse the message and a message will be logged.
