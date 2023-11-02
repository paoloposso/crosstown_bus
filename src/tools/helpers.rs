use std::{collections::BTreeMap, error::Error};

use amiquip::{QueueDeclareOptions, ExchangeDeclareOptions, ExchangeType};

pub type GenericResult = Result<(), Box<dyn Error>>;

pub(crate) fn get_exchange_name(event_name: &String) -> String {
    let mut exchange_name = event_name.to_owned();
    exchange_name.push_str("_exchange");
    exchange_name
}

pub(crate) fn create_dead_letter_policy(queue_name: String, channel: &amiquip::Channel) -> Result<String, Box::<dyn Error>>{
    let dl_ex_name = get_dead_letter_ex_name(&queue_name);
    let dl_exchange = channel.exchange_declare(amiquip::ExchangeType::Fanout, dl_ex_name.to_owned(), ExchangeDeclareOptions::default())?;
    let mut dl_queue_name = queue_name.to_owned();
    dl_queue_name.push_str(&"_dead_letter");
    let dl_queue = channel.queue_declare(dl_queue_name, QueueDeclareOptions { durable: true, exclusive: false, auto_delete: false, arguments: BTreeMap::default() })?;
    _ = dl_queue.bind(&dl_exchange, "".to_owned(), BTreeMap::default());

    Ok(dl_ex_name)
}

pub(crate) fn create_exchange<'a>(exchange_name: &'a String, exchange_type: String, channel: &'a amiquip::Channel) -> amiquip::Exchange<'a> {
    let ex_type;
    if exchange_type == "fanout" {
        ex_type = ExchangeType::Fanout;
    } else {
        ex_type = ExchangeType::Direct;
    }
    let exchange_name = exchange_name;
    let exchange = channel.exchange_declare(ex_type, exchange_name, ExchangeDeclareOptions::default()).unwrap();
    exchange
}

fn get_dead_letter_ex_name(event_name: &String) -> String {
    let mut exchange_name = event_name.to_owned();
    exchange_name.push_str("_exchange_dead_letter");
    exchange_name
}