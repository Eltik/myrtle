use crate::types::{HotFile, HotGroup};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HotUpdateResponse {
    pack_infos: Vec<PackInfo>,
    ab_infos: Vec<AbInfo>,
}

#[derive(Deserialize)]
struct PackInfo {
    name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AbInfo {
    name: String,
    total_size: u64,
    md5: String,
    pid: Option<String>,
}

/// # Errors
///
/// Returns an error if the HTTP request for `hot_update_list.json` fails or its
/// body cannot be deserialized into the expected structure.
pub async fn fetch_hot_update_list(
    client: &reqwest::Client,
    cdn_base_url: &str,
    res_version: &str,
) -> anyhow::Result<Vec<HotGroup>> {
    let url = format!("{cdn_base_url}/{res_version}/hot_update_list.json");
    let resp: HotUpdateResponse = client.get(&url).send().await?.json().await?;

    let mut groups: HashMap<String, HotGroup> = HashMap::new();
    for pack in &resp.pack_infos {
        let key = pack.name.replace('_', "/");
        groups.insert(
            key.clone(),
            HotGroup {
                name: key,
                total_size: 0,
                files: Vec::new(),
            },
        );
    }

    let mut other = HotGroup {
        name: "other".into(),
        total_size: 0,
        files: Vec::new(),
    };

    for ab in resp.ab_infos {
        let file = HotFile {
            name: ab.name,
            total_size: ab.total_size,
            md5: ab.md5,
        };

        let target = ab
            .pid
            .as_deref()
            .map(|pid| pid.replace('_', "/"))
            .and_then(|pid| groups.get_mut(&pid))
            .unwrap_or(&mut other);

        target.total_size += file.total_size;
        target.files.push(file);
    }

    let mut result: Vec<HotGroup> = groups.into_values().collect();
    if !other.files.is_empty() {
        result.push(other);
    }
    result.sort_by_key(|g| std::cmp::Reverse(g.total_size));
    Ok(result)
}
