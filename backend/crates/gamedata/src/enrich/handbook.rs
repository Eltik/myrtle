use myrtle_gamedata_raw::tables::handbook::{Handbook, HandbookItem, OperatorProfile};

use super::profile::parse_operator_profile;

#[must_use]
pub fn get_handbook_and_profile(
    char_id: &str,
    handbook: &Handbook,
) -> (HandbookItem, Option<OperatorProfile>) {
    match handbook.handbook_dict.get(char_id) {
        Some(item) => {
            let profile = parse_operator_profile(&item.story_text_audio);
            (item.clone(), profile)
        }
        None => (HandbookItem::default(), None),
    }
}
