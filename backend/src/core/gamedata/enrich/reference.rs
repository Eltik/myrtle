//! Facts a server's own wording does not carry, read from another server by id.
//!
//! Several fields are classified from EN or CN text: the profile labels, an
//! operator's `itemObtainApproach`, a skin's `displayTagId`. KR and JP word
//! them in Korean and Japanese, so the classifiers read nothing there. The
//! facts themselves are the same on every server, so this pass copies them
//! from a reference server's entry with the same id. It runs once per
//! non-default server after every server has loaded (`app::state`), and on a
//! hot reload of one (`core::asset_watcher`), with the other loaded servers as
//! references, the default first.

use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::obtain::{ObtainChannel, SkinChannel};
use crate::utils::env::switched_off;

use super::profile::align_profiles;

/// How many entries of each kind took their fact from a reference.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReferenceFacts {
    pub profiles: usize,
    pub operator_channels: usize,
    pub skin_channels: usize,
}

/// Align `target` against `references`, tried in order.
///
/// Profiles: see `profile::align_profiles`, off with `PROFILE_ALIGN=0`.
///
/// Obtain channels, off with `OBTAIN_CHANNEL_ALIGN=0`: an operator or skin
/// the target's own wording classified as `Other` takes the first reference
/// classification that says more. For an operator that is anything but
/// `Other` (an empty approach is `None` on the target and stays `None`: the
/// table's null is the same signal in every language). For a skin it is
/// anything but `Other` and `Store`, because the target carries a tag, which a
/// reference reading `Store` (no tag) contradicts. A server whose wording the
/// classifiers read keeps its own reading, so EN and CN are unchanged: on
/// 2026-10-06 CN's only `Other` operators are 主题曲剧情 and 周年奖励, which EN
/// also reads as `Other`, and CN has no `Other` skin.
pub fn align_reference_facts(target: &mut GameData, references: &[&GameData]) -> ReferenceFacts {
    let profiles = align_profiles(target, references);
    if switched_off("OBTAIN_CHANNEL_ALIGN") {
        return ReferenceFacts {
            profiles,
            ..ReferenceFacts::default()
        };
    }
    ReferenceFacts {
        profiles,
        operator_channels: align_operator_channels(target, references),
        skin_channels: align_skin_channels(target, references),
    }
}

fn align_operator_channels(target: &mut GameData, references: &[&GameData]) -> usize {
    let mut aligned = 0;
    for (id, op) in &mut target.operators {
        if op.obtain_channel != Some(ObtainChannel::Other) {
            continue;
        }
        let found = references.iter().find_map(|reference| {
            reference
                .operators
                .get(id)?
                .obtain_channel
                .filter(|c| *c != ObtainChannel::Other)
        });
        if let Some(channel) = found {
            op.obtain_channel = Some(channel);
            aligned += 1;
        }
    }
    aligned
}

fn align_skin_channels(target: &mut GameData, references: &[&GameData]) -> usize {
    let mut aligned = 0;
    let skins = &mut target.skins;
    for (id, skin) in &mut skins.char_skins {
        if skin.obtain_channel != SkinChannel::Other {
            continue;
        }
        let found = references.iter().find_map(|reference| {
            let channel = reference.skins.char_skins.get(id)?.obtain_channel;
            (!matches!(channel, SkinChannel::Other | SkinChannel::Store)).then_some(channel)
        });
        if let Some(channel) = found {
            skin.obtain_channel = channel;
            // The enriched map holds its own copy of the skin.
            if let Some(enriched) = skins.enriched_skins.get_mut(id) {
                enriched.skin.obtain_channel = channel;
            }
            aligned += 1;
        }
    }
    aligned
}
