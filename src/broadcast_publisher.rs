use std::{cell::{RefCell}};

use amiquip::{Connection, Publish};
use borsh::{BorshSerialize, BorshDeserialize};

use crate::tools::helpers::{GenericResult, create_exchange, get_exchange_name};

pub struct BroadcastPublisher(
    pub RefCell<Connection>
);

impl BroadcastPublisher {
    pub fn publish_event<TM>(&mut self, event_name: String, message: TM)
        -> GenericResult 
            where TM: BorshSerialize + BorshDeserialize {
        let mut buffer = Vec::new();
        message.serialize(&mut buffer)?;
        if let Ok(channel) = self.0.get_mut().open_channel(None) {
            let exchange_name = get_exchange_name(&event_name);
            let _ = create_exchange(&exchange_name, "fanout".to_owned(), &channel);
            let publish_result = channel.basic_publish::<String>(
                exchange_name.to_owned(),
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