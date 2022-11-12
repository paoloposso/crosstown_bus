use std::cell::Cell;

use amiquip::{Connection, Publish};
use borsh::{BorshSerialize, BorshDeserialize};

use crate::queue_subscriber::GenericResult;

pub struct QueuePublisher {
    cnn: Cell<Connection>
}

impl QueuePublisher {
    pub fn publish_event<TM>(&mut self, event_name: String, message: TM)
        -> GenericResult 
            where TM: BorshSerialize + BorshDeserialize {
        let mut buffer = Vec::new();
        message.serialize(&mut buffer)?;
        if let Ok(channel) = self.cnn.get_mut().open_channel(None) {
            let publish_result = channel.basic_publish::<String>(
                "".to_owned(),
                Publish {
                    body: &buffer,
                    routing_key: event_name.to_owned(),
                    mandatory: false,
                    immediate: false,
                    properties: Default::default(),
                });
            if publish_result.is_err() {
                return Err(Box::new(publish_result.unwrap_err()));
            }
        }
        Ok(())
    }
}