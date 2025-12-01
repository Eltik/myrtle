use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawOperator {
    name: String,
    description: String,
    can_use_general_potential_item: bool,
    can_use_activity_potential_item: bool,
    potential_item_id: String,
    activity_potential_item_id: Option<String>,
    classic_potential_item_id: Option<String>,
    nation_id: String,
    group_id: Option<String>,
    team_id: Option<String>
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operator {}
