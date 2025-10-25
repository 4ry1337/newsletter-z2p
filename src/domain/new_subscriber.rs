use crate::domain::{SubscriberEmail, SubscriberName};

// new-type pattern
pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name:  SubscriberName
}
