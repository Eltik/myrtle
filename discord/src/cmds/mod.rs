use crate::types::{Data, Error};

pub mod admin;
pub mod api;
pub mod assets;
pub mod general;

#[must_use]
pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![
        general::ping(),
        general::say(),
        admin::embed(),
        admin::ban_user(),
        admin::unban_user(),
        admin::kick_user(),
        admin::autorole(),
        admin::antispam(),
        admin::reactionrole(),
        api::api(),
        assets::assets(),
    ]
}
