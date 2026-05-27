use crate::types::{Data, Error};

pub mod general;

pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![general::ping(), general::say()]
}
