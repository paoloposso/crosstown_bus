# Rust Event Bus
Flexible and easy to configure Event Bus for event-driven systems in Rust.

Create your own structs and integrate your services / microservices by sending strongly typed payloads.

## Creating a Handler
A message received from the queue will be sent to a Handler.

Your Message Handler must implement the _trait_ MessageHandler, and inform the type of Message it you handle.

``` Rust
pub struct NotifyUserHandler {
    received_messages: Arc<Mutex<Vec<UserCreatedMessage>>>,
}

impl NotifyUserHandler {
    pub fn new(received_messages: Arc<Mutex<Vec<UserCreatedMessage>>>) -> Self {
        Self { received_messages }
    }
}

impl MessageHandler<UserCreatedMessage> for NotifyUserHandler {
    fn handle(&self, message: Box<UserCreatedMessage>) -> Result<(), HandleError> {
        self.received_messages.lock().unwrap().push(*message.clone());

        if message.user_id == "100".to_owned() {
            return Err(HandleError::new("ID 100 rejected on NotifyUserHandler".to_owned(), false));
        }
        println!("Message received on NotifyUserHandler: {:?}", message);

        Ok(())
    }
}
```

This method will receive the messages with the type that was configured, from the queue the subscriber will be listening.

The **HandleError** struct is used to inform that the process didn't ocurr corretly, though the message was received.

With the Error object you can also tell the Bus whether this message should be requeued in order to try the process again.

## Creating RabbitMQ instance on container to test locally
On terminal, run `make up`
Go to http://localhost:15672/
Enter guest as both username and password.

In your terminal, run `cargo test` to run the tests.

You can check out the exchanges, queues and messages on the RabbitMQ admin page.


## Code examples
The file `send_receive_integration_test` contains useful examples of sending messages and subscribing to exchanges to receive messages.

## Creating an Event Message
Notice that the Message type we want to send and receive between services is **UserCreatedMessage**.

When we create the Event Message struct and the Message Handler, we are defining:
- the format of the event message we are sending between services.
- the struct/method that will handle the incoming messages.

Here you have an example of Message, that we are using in the current example:

```
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct UserCreatedMessage {
    pub user_id: String,
    pub user_name: String,
    pub user_email: String,
}
```

Notice that your struct must derive from BorshDeserialize and BorshSerialize, so Crosstow Bus is able to serialize the struct you defined to send to RabbitMQ and desserialize the messages coming from RabbitMQ into your customized format.

So, don't forget to add the imports to youw cargo.toml file.

```
borsh = "1.4.0"
borsh-derive = "1.4.0"
```

## Linstening to an Event Message
First, let's create a Receiver object

```
let subscriber = CrosstownBus::new_subscriber("amqp://guest:guest@localhost:5672".to_owned())?;
```

After that, call the subscribe_event method, passing the event name / queue name that you want to subbscribe to.
If the queue was not created on RabbitMQ, it will be created when the receiver subscribes to it.

``` Rust
subscriber.subscribe(
        "user_created".to_owned(),
        NotifyUserHandler::new(received_messages.clone()),
        QueueProperties {
            auto_delete: false,
            durable: false,
            use_dead_letter: true,
            consume_queue_name: Some("queue2".to_string()),
        },
    )?;

```
Please Note that the _subscribe_event_ method in async, therefore, I'm calling _await_ when invoking it.
Another option is to block it, by using the following notation:
```
futures::executor::block_on(receiver.receive("notify_user".to_owned(), NotifyUserEventHandler, None));
```

The parameter queue_properties is optional and holds specific queue configurations, such as whether the queue should be auto deleted and durable.

### !!! Important: consume_queue_name !!!

When you subscribe to an event, the subscriber has its own queue.

The publisher publishes to an exchange, *not to a queue*.

Every queue receives a copy of the message.

Subscribers may have, each of them, a different queue. All these queues will receive a copy of the message.

Subscribers that use the same queue, will compete for the messages so these messages will be distributed among them. The outcome is that they will not receive all the messages every time.

So, if you have multiple subscribers that need to handle the same messages, choose a different queue name for each of them.

## Publishing / sending an Event Message
To create the sender the process is pretty much the same, only a different creation method.

``` Rust
let mut publisher = CrosstownBus::new_publisher("amqp://guest:guest@localhost:5672".to_owned())?;

_ = publisher.publish("notify_user".to_owned(), 
        UserCreatedMessage {
            user_id: "asdf".to_owned(),
            user_name: "Billy Gibbons".to_owned(),
            email: "bg@test.com".to_owned()
        });
```
Since the method send receives a generic parameter as the Message, we can use the same sender object to publish multiple objects types to multiple queues.
**Warning:** if the message type you are publishing on a queue doesn't match what the subscriber handler is expecting, it will not be possible to parse the message and a message will be logged.

## Dead Letter Exchange
Dead letter exchanges are useful when you want to handle messages that were not processed correctly.

When subscribing to an event, you can set the use_dead_letter property to true, so that the messages that were not processed correctly are sent to a dead letter exchange.
```Rust
            use_dead_letter: true,
```

### Subscribing to the Dead Letter Exchange
When the handler fails on trying to process the message, the message is sent to the dead letter exchange.

You can create a handler to process these messages, and subscribe to the dead letter exchange.

```Rust
dl_subscriber.subscribe(
        "insert_user.dlx".to_owned(),
        AddUserToDBDeadLetterHandler::new(received_messages.clone()),
        QueueProperties {
            auto_delete: false,
            durable: false,
            use_dead_letter: false,
            consume_queue_name: Some("handle_insert_user_dl".to_string()),
        },
    )?;
```

## Closing the connection to RabbitMQ

You can also manually close the connection, if needed:

```
_ = publisher.close_connection();
```