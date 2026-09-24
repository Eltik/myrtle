//! The Story Collection's own art: what the client draws beside a shelf, an
//! arc header and a chapter card.
//!
//! The screen is `ui/[uc]mixstory.ab` (`storyline_view`, `split_view`,
//! `act_view`), and every image it draws ships in the `spritepack/mixstory_*`
//! bundles as a plain `Texture2D` + full-rect `Sprite` pair, one texture per
//! sprite. Nothing here is atlas-packed, so the ordinary texture pass already
//! writes each one as `textures/spritepack/<bundle>/<m_Name>.png` and no rect
//! has to be cut out of a page.
//!
//! `stage_table` addresses them by id and this is where each id lives:
//! `StorylineStorySets[].KvImageId` (`kv_*`) in `mixstory_kv_sprites_{0,1,2}`,
//! `TitleImageId` (`title_*`) in `mixstory_title_sprites_0`,
//! `MainlineData.DecoImageId` (`deco_*`) and a `MAINLINE_SPLIT` location's
//! `MainlineSplitData.IconId` (`act_0`..`act_3`) in
//! `mixstory_deco_sprites_h2_0`, `Storylines[].StorylineIconId`
//! (`storyline_abbr_*`) in `mixstory_abbr_sprites_h2_0`, `StorylineLogoId`
//! (`storyline_*`) in `mixstory_logo_sprites_0`, and the shelf backgrounds
//! (`bg_*`, `storybg_*`) in `mixstory_background_sprites_0` and
//! `mixstory_retro_bkg_sprites_{0..5}`.
//!
//! `unpacker backfill-story-art` writes only these bundles into an existing
//! output tree, the way `backfill-hubs` and `backfill-sprites` do, so the art
//! can be refreshed without a multi-hour re-extract.

use std::path::Path;

/// True for a bundle whose textures are Story Collection art: the
/// `spritepack/mixstory_*` family and nothing else.
#[must_use]
pub fn is_story_art_bundle(bundle_subdir: &Path) -> bool {
    let mut it = bundle_subdir.components();
    let first = it.next().and_then(|c| c.as_os_str().to_str());
    let second = it.next().and_then(|c| c.as_os_str().to_str());
    matches!((first, second), (Some("spritepack"), Some(sub)) if sub.starts_with("mixstory_"))
        && it.next().is_none()
}

/// Which `stage_table` field a written PNG answers, by its stem. Used by the
/// backfill's report so the counts line up with the wire fields.
#[must_use]
pub fn art_kind(stem: &str) -> &'static str {
    if stem.starts_with("kv_") {
        "kv (StorylineStorySets.KvImageId)"
    } else if stem.starts_with("title_") {
        "title (StorylineStorySets.TitleImageId)"
    } else if stem.starts_with("storyline_abbr_") {
        "storyline glyph (Storylines.StorylineIconId)"
    } else if stem.starts_with("storyline_") {
        "storyline logo (Storylines.StorylineLogoId)"
    } else if stem.starts_with("deco_") {
        "chapter deco (MainlineData.DecoImageId)"
    } else if stem.starts_with("act_") {
        "arc icon (MainlineSplitData.IconId)"
    } else {
        "shelf background (BackgroundId)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_mixstory_spritepacks_are_story_art() {
        for ok in [
            "spritepack/mixstory_kv_sprites_0",
            "spritepack/mixstory_abbr_sprites_h2_0",
            "spritepack/mixstory_deco_sprites_h2_0",
            "spritepack/mixstory_retro_bkg_sprites_5",
        ] {
            assert!(is_story_art_bundle(Path::new(ok)), "{ok}");
        }
        for no in [
            "spritepack/story_review_chapter_bg_h1_0",
            "spritepack/ui_kv_img_0",
            "ui/zonemap_0",
            "spritepack",
            "spritepack/mixstory_kv_sprites_0/nested",
        ] {
            assert!(!is_story_art_bundle(Path::new(no)), "{no}");
        }
    }

    #[test]
    fn a_stem_names_the_field_it_answers() {
        assert!(art_kind("kv_evil_time_part1").starts_with("kv"));
        assert!(art_kind("title_evil_time_part1").starts_with("title"));
        assert!(art_kind("storyline_abbr_Rl").starts_with("storyline glyph"));
        assert!(art_kind("storyline_Ms").starts_with("storyline logo"));
        assert!(art_kind("deco_evil_time_part1").starts_with("chapter deco"));
        assert!(art_kind("act_0").starts_with("arc icon"));
        assert!(art_kind("bg_mainLine_0").starts_with("shelf background"));
    }
}
