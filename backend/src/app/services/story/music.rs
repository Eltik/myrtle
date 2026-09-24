//! The theme a story group plays over its Archives entry.
//!
//! What this module OWNS is the join from a group id to a `Musics` row, a BGM
//! bank and the two clips on disk. It is one function because the rule is one
//! chain of fallbacks, each step derived from the rows the step before it
//! missed; the audit trail for those steps is the doc comment below and
//! `docs/story-reader.md`, section "1. What is true about the data".

use super::dto::{StoryCategory, StoryMusic};
use super::opt;
use crate::core::gamedata::types::GameData;
use crate::core::story::StoryAssetIndex;

/// The theme a group plays, `None` when nothing names one or its loop clip is
/// not on disk.
///
/// The rule the audit stated reaches 77 of the 87 EN library groups: a
/// mainline chapter through `music_3in1bg_main{N}` and an event through the
/// `sys.ON_ACTIVITY_LOADED.{id}` bank. The other TEN have a theme too and are
/// reached by three further steps, each derived from the rows that were
/// missed: a bank with the group id plus a suffix (`act27side.day`,
/// `act44side.theme1`, `act9mini.0`), a `music_3in1bg_{id}` row whose bank is
/// a retro one (`1stact`, `act3d0`), and a bank with no `Musics` row at all
/// (`act17d0`, `act24side`, `act32side`, `act36side`, `act13mini`), which
/// gives a track with no title. That takes it to 86 of 87; `act24side`'s loop
/// clip is the one the tree does not hold.
pub(super) fn group_music(
    gd: &GameData,
    story_assets: &StoryAssetIndex,
    group_id: &str,
    category: StoryCategory,
) -> Option<StoryMusic> {
    let banks = &gd.music;
    let music = if category == StoryCategory::Main {
        let n = group_id.strip_prefix("main_")?;
        banks.music(&format!("music_3in1bg_main{n}"))
    } else {
        let exact = format!("sys.ON_ACTIVITY_LOADED.{group_id}");
        banks
            .music_for_bank(&exact)
            .or_else(|| banks.music_by_bank_prefix(&format!("{exact}.")))
            .or_else(|| banks.music(&format!("music_3in1bg_{group_id}")))
    };
    let bank_name = music.map_or_else(
        || format!("sys.ON_ACTIVITY_LOADED.{group_id}"),
        |m| m.bank.clone(),
    );
    let bank = banks.bank(&bank_name)?;
    let loop_url = story_assets.resolve_audio(bank.loop_.as_deref()?)?;
    Some(StoryMusic {
        title: music.and_then(|m| opt(&m.name)),
        intro_url: bank
            .intro
            .as_deref()
            .and_then(|i| story_assets.resolve_audio(i)),
        loop_url,
    })
}
