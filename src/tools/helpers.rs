use std::error::Error;

use amiquip::{ExchangeDeclareOptions, ExchangeType};

pub type GenericResult = Result<(), Box<dyn Error>>;

pub(crate) fn create_exchange<'a>(exchange_name: &'a String, exchange_type: String, channel: &'a amiquip::Channel) -> amiquip::Exchange<'a> {
    let ex_type;
    if exchange_type == "fanout" {
        ex_type = ExchangeType::Fanout;
    } else {
        ex_type = ExchangeType::Direct;
    }
    let exchange = channel.exchange_declare(ex_type, exchange_name, ExchangeDeclareOptions::default()).unwrap();
    exchange
}

pub(crate) fn get_dead_letter_ex_name(event_name: &String) -> String {
    let mut exchange_name = event_name.to_owned();
    exchange_name.push_str(".dlx");
    exchange_name
}