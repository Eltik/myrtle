use serde::{Deserialize, Serialize};
#[cfg(feature = "schema")]
use utoipa::ToSchema;
use uuid::Uuid;

macro_rules! uuid_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[cfg_attr(feature = "schema", derive(ToSchema))]
        #[serde(transparent)]
        pub struct $name(pub Uuid);
    };
}

macro_rules! string_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[cfg_attr(feature = "schema", derive(ToSchema))]
        #[serde(transparent)]
        pub struct $name(pub Box<str>);

        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

uuid_id!(
    /// `users.id`
    UserId
);
uuid_id!(
    /// `tier_lists.id`
    TierListId
);
uuid_id!(
    /// `grids.id`
    GridId
);
uuid_id!(
    /// `user_squads.id`
    SquadId
);
uuid_id!(
    /// `sessions.id`
    SessionId
);
uuid_id!(
    /// `api_keys.id`
    ApiKeyId
);
uuid_id!(
    /// `worker_heartbeats.worker_id`
    WorkerId
);

/// `jobs.id`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ToSchema))]
#[serde(transparent)]
pub struct JobId(pub i64);

string_id!(
    /// users.uid
    GameUid
);
string_id!(
    /// `char_002_amiya`
    OperatorId
);
string_id!(
    /// `char_002_amiya@epoque#4`
    SkinId
);
string_id!(
    /// `main_01-07`
    StageId
);
string_id!(
    /// Item id from `item_table`
    ItemId
);
string_id!(
    /// Medal id from `medal_table`
    MedalId
);
string_id!(
    /// Enemy id from `enemy_handbook_table`
    EnemyId
);
string_id!(
    /// Story id from `story_review_table`
    StoryId
);
