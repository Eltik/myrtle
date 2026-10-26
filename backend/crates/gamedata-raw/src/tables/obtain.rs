use serde::{Deserialize, Serialize};
#[cfg(feature = "schema")]
use utoipa::ToSchema;

/// Operator's `itemObtainApproach` category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(ToSchema))]
pub enum ObtainChannel {
    Headhunting,
    Event,
    Voucher,
    IntegratedStrategies,
    CreditStore,
    GiftPack,
    /// Non-empty approach outside the other categories, e.g. "Main Theme Story"
    Other,
}

impl ObtainChannel {
    /// EN and CN wording, by prefix. `None` for an empty approach
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

/// Skin's shop channel, from `displaySkin.displayTagId`
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(ToSchema))]
pub enum SkinChannel {
    /// No tag: outfit store and default outfits
    #[default]
    Store,
    Collab,
    Event,
    IntegratedStrategies,
    Seasonal,
    SpecialPack,
    /// EN "Official Artworks", CN 设定集: a code inside the artbook
    CodeExchange,
    ReclamationAlgorithm,
    /// Unrecognized tag
    Other,
}

impl SkinChannel {
    /// EN and CN `displayTagId` wording, exact match
    pub fn classify(display_tag_id: Option<&str>) -> Self {
        let Some(tag) = display_tag_id.map(str::trim).filter(|t| !t.is_empty()) else {
            return Self::Store;
        };
        match tag {
            "From collabs" | "联动获得" => Self::Collab,
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
