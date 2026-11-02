use myrtle_types::Server;

use crate::model::GameVersion;

#[derive(Debug, Clone)]
pub struct CompileOptions {
    pub server: Server,
    pub game_version: GameVersion,
    pub profile_align: bool,
    pub obtain_channel_align: bool,
}
