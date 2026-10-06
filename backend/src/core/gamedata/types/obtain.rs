//! How an operator or a skin is obtained, as a category instead of the game's
//! localized wording.
//!
//! The tables name the category in the server's own language
//! (`itemObtainApproach` "Recruitment & Headhunting" on EN is 招募寻访 on CN,
//! 인재 채용 on KR and 人材募集 on JP), so a client that groups by the string
//! only works on the language it was written against. The classifiers here
//! read EN and CN wording; KR and JP take the category from a reference
//! server by id, see `enrich::reference::align_reference_facts`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// An operator's `itemObtainApproach` category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub enum ObtainChannel {
    Headhunting,
    Event,
    Voucher,
    IntegratedStrategies,
    CreditStore,
    GiftPack,
    /// A non-empty approach outside the six categories: EN "Main Theme Story"
    /// and "Anniversary Reward", one operator each on 2026-10-06, or wording
    /// the classifier does not read.
    Other,
}

impl ObtainChannel {
    /// EN and CN wording, by prefix: EN's one combined value is
    /// "Recruitment & Headhunting, Pinboard Missions" (CN 招募寻访、见习任务)
    /// and it reads as headhunting. `None` for an empty approach, which is how
    /// the enrichment stores the table's null (29 operators on every server).
    pub fn classify(approach: &str) -> Option<Self> {
        const PREFIXES: &[(&str, ObtainChannel)] = &[
            ("Recruitment & Headhunting", ObtainChannel::Headhunting),
            ("招募寻访", ObtainChannel::Headhunting),
            ("Event Reward", ObtainChannel::Event),
            ("活动获得", ObtainChannel::Event),
            ("Voucher Exchange", ObtainChannel::Voucher),
            ("凭证交易所", ObtainChannel::Voucher),
            (
                "Obtained from Integrated Strategies",
                ObtainChannel::IntegratedStrategies,
            ),
            ("集成战略", ObtainChannel::IntegratedStrategies),
            ("Credit Store", ObtainChannel::CreditStore),
            ("信用交易所", ObtainChannel::CreditStore),
            ("Limited Gift Pack", ObtainChannel::GiftPack),
            ("限时礼包", ObtainChannel::GiftPack),
        ];
        let approach = approach.trim();
        if approach.is_empty() {
            return None;
        }
        Some(
            PREFIXES
                .iter()
                .find(|(prefix, _)| approach.starts_with(prefix))
                .map_or(Self::Other, |(_, channel)| *channel),
        )
    }
}

/// A skin's shop channel, from `displaySkin.displayTagId`. The categories are
/// the ones the profile skin viewer groups by.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub enum SkinChannel {
    /// No tag: the outfit store, and the default outfits. The absence of a tag
    /// reads the same in every language, so every server classifies this one
    /// itself.
    #[default]
    Store,
    Collab,
    Event,
    IntegratedStrategies,
    Seasonal,
    SpecialPack,
    /// EN "Official Artworks", CN 设定集: a code inside the artbook.
    CodeExchange,
    ReclamationAlgorithm,
    /// A tag the classifier does not read, with no reference to resolve it.
    Other,
}

impl SkinChannel {
    /// EN and CN `displayTagId` wording, exact. Census of 2026-10-06: EN 144
    /// tagged skins over 7 tags, CN 161 over 8.
    pub fn classify(display_tag_id: Option<&str>) -> Self {
        let Some(tag) = display_tag_id.map(str::trim).filter(|t| !t.is_empty()) else {
            return Self::Store;
        };
        match tag {
            "From collabs" | "联动获得" => Self::Collab,
            // The two KFC skins, CN only: a co-branded pack, not a paid brand.
            "合作礼包" => Self::Collab,
            "Event Reward" | "活动获得" => Self::Event,
            "Integrated Strategies" | "集成战略" => Self::IntegratedStrategies,
            "Seasonal Attire" | "赛季服饰" => Self::Seasonal,
            "Obtain from Special Pack" | "特典获得" => Self::SpecialPack,
            "Official Artworks" | "设定集" => Self::CodeExchange,
            "Reclamation Algorithm" | "生息演算" => Self::ReclamationAlgorithm,
            _ => Self::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operator_wording_reads_by_prefix() {
        assert_eq!(
            ObtainChannel::classify("Recruitment & Headhunting, Pinboard Missions"),
            Some(ObtainChannel::Headhunting)
        );
        assert_eq!(
            ObtainChannel::classify("集成战略获得"),
            Some(ObtainChannel::IntegratedStrategies)
        );
        assert_eq!(
            ObtainChannel::classify("Main Theme Story"),
            Some(ObtainChannel::Other)
        );
        // KR wording is not read here; the reference pass resolves it.
        assert_eq!(
            ObtainChannel::classify("인재 채용"),
            Some(ObtainChannel::Other)
        );
        assert_eq!(ObtainChannel::classify(""), None);
    }

    #[test]
    fn skin_tags_read_exactly() {
        assert_eq!(SkinChannel::classify(None), SkinChannel::Store);
        assert_eq!(SkinChannel::classify(Some("联动获得")), SkinChannel::Collab);
        assert_eq!(
            SkinChannel::classify(Some("Official Artworks")),
            SkinChannel::CodeExchange
        );
        assert_eq!(SkinChannel::classify(Some("콜라보")), SkinChannel::Other);
    }
}
