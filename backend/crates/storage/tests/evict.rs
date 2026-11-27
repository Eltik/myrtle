//! `EvictPolicy` parsing, the keep list, and in-place truncation.

use std::collections::HashMap;

use myrtle_storage::config::ConfigError;
use myrtle_storage::evict::{EvictPolicy, truncate_in_place};

fn policy(pairs: &[(&str, &str)]) -> Result<EvictPolicy, ConfigError> {
    let map: HashMap<String, String> = pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    EvictPolicy::from_lookup(&|k| map.get(k).cloned())
}

#[test]
fn eviction_is_off_unless_switched_on() {
    assert_eq!(policy(&[]), Ok(EvictPolicy::default()));
    for off in ["", "  ", "0", "false", "no", "off", "enable", "2"] {
        assert!(
            !policy(&[("STORAGE_EVICT_LOCAL", off)]).unwrap().enabled,
            "{off:?}"
        );
    }
    for on in ["1", "true", "TRUE", "yes", "On", " on "] {
        assert!(
            policy(&[("STORAGE_EVICT_LOCAL", on)]).unwrap().enabled,
            "{on:?}"
        );
    }
}

#[test]
fn extra_keep_prefixes_parse() {
    let p = policy(&[("STORAGE_EVICT_KEEP", " /textures/ui/ , ,audio/bgm/x.m4a,")]).unwrap();
    assert_eq!(p.extra_keep, ["textures/ui/", "audio/bgm/x.m4a"]);
    assert_eq!(
        policy(&[("STORAGE_EVICT_KEEP", "")]).unwrap().extra_keep,
        Vec::<String>::new()
    );
    assert!(matches!(
        policy(&[("STORAGE_EVICT_KEEP", "textures\\avg")]),
        Err(ConfigError::Invalid {
            var: "STORAGE_EVICT_KEEP",
            ..
        })
    ));
}

#[test]
fn the_keep_list_matches_by_prefix() {
    let p = EvictPolicy::default();
    for kept in [
        "textures/avg/characters/x.png",
        "textures/avg/hub.json",
        "textures/spritepack/cutin_char_amiya_1.png",
        "textures/spritepack/ui_home_act_banner_gacha_1.png",
        "textures/spritepack/ui_zone_home_theme_2.png",
        "textures/arts/ui/stage/[uc]homeentry/a.png",
        "textures/spritepack/mixstory_kv_0/kv_act17side.png",
        "audio/m4a_sources.json",
        "textures/[[en]]/arts/ui/homebanners/a.png",
        "textures/[[cn]]/arts/ui/homebanners/sub/b.png",
    ] {
        assert!(p.keeps(kept), "{kept}");
    }
    for evicted in [
        "textures/avgx/a.png",
        "textures/spritepack/ui_other.png",
        "textures/arts/ui/stage/[uc]mappreviewsspecial/a.png",
        "spine/DynIllust/x/[scene].json",
        "audio/sound_beta_2/m.m4a",
        "portraits/a.png",
        "textures/[[en]]/arts/ui/other/a.png",
        "textures/[[en]]/x/arts/ui/homebanners/a.png",
        "textures/arts/ui/homebanners/a.png",
        "textures/[[en]arts/ui/homebanners/a.png",
        "spine/[[en]]/arts/ui/homebanners/a.png",
    ] {
        assert!(!p.keeps(evicted), "{evicted}");
    }
    let extra = EvictPolicy {
        enabled: true,
        extra_keep: vec!["portraits/".into()],
    };
    assert!(extra.keeps("portraits/a.png"));
    assert!(extra.keeps("textures/avg/a.png"));
}

#[test]
fn truncation_preserves_the_mtime() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("a.png");
    std::fs::write(&path, b"bytes").unwrap();
    let old = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000);
    std::fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_modified(old)
        .unwrap();
    let before = std::fs::metadata(&path).unwrap().modified().unwrap();
    truncate_in_place(&path).unwrap();
    let after = std::fs::metadata(&path).unwrap();
    assert_eq!(after.len(), 0);
    assert_eq!(after.modified().unwrap(), before);
    assert_eq!(before, old);
}

#[test]
fn truncation_keeps_the_inode() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("a.png");
    std::fs::write(&path, b"bytes").unwrap();
    #[cfg(unix)]
    let ino = {
        use std::os::unix::fs::MetadataExt;
        std::fs::metadata(&path).unwrap().ino()
    };
    truncate_in_place(&path).unwrap();
    let meta = std::fs::metadata(&path).unwrap();
    assert_eq!(meta.len(), 0);
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        assert_eq!(meta.ino(), ino);
    }
    assert!(truncate_in_place(&dir.path().join("missing.png")).is_err());
    assert!(!dir.path().join("missing.png").exists());
}
