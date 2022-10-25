
use std::error::Error;

use amiquip::Publish;
use borsh::BorshSerialize;
use amiquip::Connection;
use borsh::BorshDeserialize;

use crate::{tools::helpers::get_event_name, message::Message};

pub struct Publisher(Box::<Connection>);

impl Publisher {
    pub fn new(url: String) -> Result<Self, Box<dyn Error>> {
        let cnn = Connection::insecure_open(&url)?;
        Ok(Self(Box::new(cnn)))
    }
    
    pub fn publish_event<T>(&mut self, message: Message::<T>) -> Result<(), Box<dyn Error>> 
        where T: BorshDeserialize + BorshSerialize {
        let event_name = get_event_name::<T>();

        let mut buffer = Vec::new();

        message.serialize(&mut buffer)?;

        if let Ok(channel) = self.0.open_channel(None) {
            let publish_result = channel.basic_publish::<String>(
                event_name.to_owned(), 
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

    pub fn disconnect_publisher(self) -> Result<(), Box::<dyn Error>> {
        self.0.close()?;
        Ok(())
    }
}