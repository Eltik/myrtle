use myrtle_gamedata_raw::tables::{item::SkinChannel, obtain::ObtainChannel};

use crate::{compile::CompileOptions, model::GameModel};

use super::profile::align_profiles;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReferenceFacts {
    pub profiles: usize,
    pub operator_channels: usize,
    pub skin_channels: usize,
}

pub fn align(
    target: &mut GameModel,
    references: &[&GameModel],
    opts: &CompileOptions,
) -> ReferenceFacts {
    let profiles = align_profiles(target, references, opts);
    if !opts.obtain_channel_align {
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

fn align_operator_channels(target: &mut GameModel, references: &[&GameModel]) -> usize {
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

fn align_skin_channels(target: &mut GameModel, references: &[&GameModel]) -> usize {
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
            if let Some(enriched) = skins.enriched_skins.get_mut(id) {
                enriched.skin.obtain_channel = channel;
            }
            aligned += 1;
        }
    }
    aligned
}
