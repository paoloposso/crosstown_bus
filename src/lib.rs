pub mod subscriber;

#[cfg(test)]
mod tests {
    use crate::subscriber::RabbitSubscriber;

    #[test]
    fn rabbit_subscriber_new_works() {
        if let Ok(_subs) = RabbitSubscriber::new() {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
