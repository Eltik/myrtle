//! Enemy battle sounds on the real EN tree: how many handbook enemies get at
//! least one playable bank, and three enemies traced end to end.

mod common;

use backend::core::gamedata::types::audio::{EnemyAudio, EnemyAudioCategory};
use common::load_game_data;

fn clips<'a>(audio: &'a [EnemyAudio], bank: &str) -> Vec<&'a str> {
    audio
        .iter()
        .find(|a| a.bank_name == bank)
        .map(|a| {
            a.sounds
                .iter()
                .flat_map(|s| s.urls.iter().map(String::as_str))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn enemy_sounds_link_and_resolve() {
    let data = load_game_data();
    let total = data.enemies.enemy_data.len();
    let with_audio = data
        .enemies
        .enemy_data
        .keys()
        .filter(|id| data.enemy_audio.get(*id).is_some_and(|a| !a.is_empty()))
        .count();
    let direct = data
        .enemy_audio
        .values()
        .filter(|a| a.iter().any(|c| c.via.is_none()))
        .count();
    let banks: usize = data.enemy_audio.values().map(Vec::len).sum();
    eprintln!(
        "{with_audio} of {total} handbook enemies have sounds ({direct} by id or prefab key), {banks} enemy-bank links"
    );
    let mut by_category: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    for c in data.enemy_audio.values().flatten() {
        *by_category.entry(format!("{:?}", c.category)).or_default() += 1;
    }
    eprintln!("enemy-bank links by category: {by_category:?}");

    // Measured 2026-10-11 on EN: 1,590 handbook enemies.
    assert!(
        with_audio * 100 / total >= 85,
        "coverage fell: {with_audio}/{total}"
    );

    // Originium Slug: one melee hit bank, `Enemy/e_imp/e_imp_general_w`,
    // flattened on disk into the numbered `enmy_snd_imp_2` bucket.
    let slug = &data.enemy_audio["enemy_1007_slime"];
    assert_eq!(
        clips(slug, "battle.ON_ABILITY_HIT.enemy_1007_slime.combat"),
        ["/audio/sound_beta_2/enmy_snd_imp_2/e_imp_general_w.ogg"]
    );
    assert!(
        slug.iter()
            .all(|a| a.category == EnemyAudioCategory::Attack)
    );

    // Soldier: the weighted swing has two clips (the 80-weight silent slot
    // has an empty asset and is dropped).
    let gopro = &data.enemy_audio["enemy_1000_gopro"];
    assert_eq!(
        clips(gopro, "battle.ON_ABILITY_ON.enemy_1000_gopro.combat"),
        [
            "/audio/sound_beta_2/enmy_snd_atk_5/e_atk_gopro_1.ogg",
            "/audio/sound_beta_2/enmy_snd_atk_5/e_atk_gopro_2.ogg"
        ]
    );

    // A boss: its entrance, death and named skills each link by id.
    let sfsui = &data.enemy_audio["enemy_1526_sfsui"];
    let cat = |bank: &str| {
        sfsui
            .iter()
            .find(|a| a.bank_name == bank)
            .map(|a| a.category)
    };
    assert_eq!(
        cat("battle.ON_BOSS_ENTER.enemy_1526_sfsui"),
        Some(EnemyAudioCategory::Spawn)
    );
    assert_eq!(
        cat("battle.ON_UNIT_DEAD.enemy_1526_sfsui"),
        Some(EnemyAudioCategory::Death)
    );
    assert_eq!(
        cat("battle.ON_ABILITY_ON.enemy_1526_sfsui.ThunderS1"),
        Some(EnemyAudioCategory::Skill)
    );
    assert_eq!(
        cat("battle.ON_BUFF_START.enemy_sfsui_doom1"),
        Some(EnemyAudioCategory::Skill)
    );

    // FrostNova has no bank under her own id; her projectiles link by codename.
    let frstar = &data.enemy_audio["enemy_1505_frstar"];
    assert!(
        frstar
            .iter()
            .all(|a| a.category == EnemyAudioCategory::Projectile)
    );
    assert!(
        frstar
            .iter()
            .any(|a| a.via.as_deref() == Some("projectile_frstar"))
    );

    // W the enemy shares `cqbw` with W the operator: only the explicitly
    // enemy projectile links.
    let w = &data.enemy_audio["enemy_1504_cqbw"];
    assert!(
        w.iter()
            .all(|a| a.via.as_deref() != Some("projectile_cqbw"))
    );
    assert!(
        w.iter()
            .any(|a| a.via.as_deref() == Some("projectile_enemy_cqbw"))
    );
}
