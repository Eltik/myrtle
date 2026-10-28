use serde::{Deserialize, Serialize};

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

/// One lobby home-screen background
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HomeBackground {
    #[serde(alias = "BgId", alias = "bgId", default)]
    pub bg_id: String,
    /// Lobby name (`Vision`), in the server's language
    #[serde(alias = "BgName", alias = "bgName", default)]
    pub bg_name: String,
    /// Lobby order
    #[serde(alias = "BgSortId", alias = "bgSortId", default)]
    pub bg_sort_id: i32,
    /// Unix seconds, `-1` for the two defaults
    #[serde(alias = "BgStartTime", alias = "bgStartTime", default)]
    pub bg_start_time: i64,
    /// `BASE`, or `VARIABLE` for a background that changes with the time of day
    #[serde(alias = "BgType", alias = "bgType", default)]
    pub bg_type: String,
    /// How it is obtained, in the server's language (`Purchase Pack`)
    #[serde(alias = "ObtainApproach", alias = "obtainApproach", default)]
    pub obtain_approach: String,
}

impl DisplayMetaTableFile {
    /// Backgrounds in lobby order
    pub fn into_home_backgrounds(self) -> Vec<HomeBackground> {
        let mut list = self.home_background_data.home_bg_data_list;
        list.retain(|bg| !bg.bg_id.is_empty());
        list.sort_by_key(|bg| bg.bg_sort_id);
        list
    }
}
