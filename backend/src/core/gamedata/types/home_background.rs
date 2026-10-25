//! `display_meta_table.HomeBackgroundData`: the home-screen backgrounds a
//! player picks from in the lobby's Background menu.
//!
//! On the EN table of 2026-10-20 there are 28 rows, 26 `BASE` and 2
//! `VARIABLE` (the time-of-day ones, `bg_volcano_1` and `bg_lonetrail_1`); CN
//! of 2026-10-24 has 34. The lobby lists them by `BgSortId`.

use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DisplayMetaTableFile {
    #[serde(alias = "HomeBackgroundData", alias = "homeBackgroundData", default)]
    pub home_background_data: HomeBackgroundData,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct HomeBackgroundData {
    #[serde(alias = "HomeBgDataList", alias = "homeBgDataList", default)]
    pub home_bg_data_list: Vec<HomeBackground>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct HomeBackground {
    #[serde(alias = "BgId", alias = "bgId", default)]
    pub bg_id: String,
    /// The lobby's name for it (`Vision`), in the server's language.
    #[serde(alias = "BgName", alias = "bgName", default)]
    pub bg_name: String,
    /// The lobby's order.
    #[serde(alias = "BgSortId", alias = "bgSortId", default)]
    pub bg_sort_id: i32,
    /// Unix seconds; `-1` for the two defaults.
    #[serde(alias = "BgStartTime", alias = "bgStartTime", default)]
    pub bg_start_time: i64,
    /// `BASE`, or `VARIABLE` for a background that changes with the time of day.
    #[serde(alias = "BgType", alias = "bgType", default)]
    pub bg_type: String,
    /// How it is obtained, in the server's language (`Purchase Pack`,
    /// `Integrated Strategies reward`).
    #[serde(alias = "ObtainApproach", alias = "obtainApproach", default)]
    pub obtain_approach: String,
}

impl DisplayMetaTableFile {
    /// The backgrounds in the lobby's order.
    pub fn into_home_backgrounds(self) -> Vec<HomeBackground> {
        let mut list = self.home_background_data.home_bg_data_list;
        list.retain(|bg| !bg.bg_id.is_empty());
        list.sort_by_key(|bg| bg.bg_sort_id);
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backgrounds_read_in_the_lobby_order() {
        let raw = r#"{"HomeBackgroundData": {"HomeBgDataList": [
            {"BgId": "bg_rhodes_night", "BgName": "Nighttime", "BgSortId": 2,
             "BgStartTime": -1, "BgType": "BASE", "ObtainApproach": "Default"},
            {"BgId": "bg_rhodes_day", "BgName": "Vision", "BgSortId": 1,
             "BgStartTime": -1, "BgType": "BASE", "ObtainApproach": "Default"}
        ]}}"#;
        let file: DisplayMetaTableFile = serde_json::from_str(raw).expect("parses");
        let list = file.into_home_backgrounds();
        let ids: Vec<_> = list.iter().map(|bg| bg.bg_id.as_str()).collect();
        assert_eq!(ids, ["bg_rhodes_day", "bg_rhodes_night"]);
        assert_eq!(list[0].bg_name, "Vision");
        assert_eq!(list[0].obtain_approach, "Default");
    }
}
