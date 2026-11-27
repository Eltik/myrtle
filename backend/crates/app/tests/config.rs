#![allow(clippy::unwrap_used)]

use std::collections::BTreeSet;
use std::io::Write as _;

use myrtle_app::config::{Config, ConfigError, FIELDS, RETIRED, SecretRef, render_example};

const EXAMPLE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../config/myrtle.example.toml"
);

fn example() -> Config {
    toml::from_str(&render_example()).unwrap()
}

#[test]
fn example_is_current() {
    let rendered = render_example();
    if std::env::var_os("MYRTLE_BLESS").is_some() {
        std::fs::write(EXAMPLE_PATH, &rendered).unwrap();
    }
    let on_disk = std::fs::read_to_string(EXAMPLE_PATH).unwrap();
    assert_eq!(
        on_disk, rendered,
        "config/myrtle.example.toml is stale: MYRTLE_BLESS=1 cargo test -p myrtle-app example_is_current"
    );
}

#[test]
fn example_parses_into_config() {
    let config = example();
    assert_eq!(config.server.port, 3060);
    assert!(config.switches.role_recheck.enabled);
}

#[test]
fn every_legacy_env_var_appears_once() {
    let mut seen = BTreeSet::new();
    let names = FIELDS
        .iter()
        .flat_map(|f| f.env.iter().copied())
        .chain(RETIRED.iter().map(|r| r.env));
    for name in names {
        assert!(seen.insert(name), "{name} is mapped twice");
    }
}

#[test]
fn keys_are_unique() {
    let mut seen = BTreeSet::new();
    for f in FIELDS {
        assert!(seen.insert(f.key), "{} is listed twice", f.key);
    }
}

#[test]
fn every_switch_field_is_named_once() {
    let config = example();
    let named: BTreeSet<&str> = config
        .switches
        .all()
        .iter()
        .map(|(name, _)| *name)
        .collect();
    let listed: BTreeSet<&str> = FIELDS
        .iter()
        .filter_map(|f| f.key.strip_prefix("switches."))
        .collect();
    assert_eq!(named, listed);
}

#[test]
fn a_switch_expires_the_day_after_its_date() {
    let mut config = example();
    config.switches.story_hub.remove_after = "2026-03-01".into();
    let expired = |today: &str| -> Vec<&'static str> {
        config
            .expired_switches(today)
            .into_iter()
            .map(|s| s.name)
            .collect()
    };
    assert!(expired("2026-03-01").is_empty());
    assert_eq!(expired("2026-03-02"), ["story_hub"]);
}

#[test]
fn a_file_secret_loses_its_trailing_newline() {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "s3cret").unwrap();
    let secret = SecretRef(format!("file:{}", file.path().display()));
    assert_eq!(secret.resolve().unwrap(), "s3cret");
}

#[test]
fn an_empty_or_unknown_secret_is_refused() {
    let empty = tempfile::NamedTempFile::new().unwrap();
    for reference in [
        format!("file:{}", empty.path().display()),
        "file:/nonexistent/myrtle-secret".to_owned(),
        "env:MYRTLE_TEST_SECRET_THAT_IS_NOT_SET".to_owned(),
        "plain-value".to_owned(),
    ] {
        let refused = SecretRef(reference.clone()).resolve();
        assert!(
            matches!(&refused, Err(ConfigError::Secret { reference: r }) if *r == reference),
            "{reference}: {refused:?}"
        );
    }
}

#[test]
fn a_bad_switch_date_fails_validation() {
    let mut config = example();
    config.switches.art_fallback.remove_after = "next spring".into();
    let refused = config.validate();
    assert!(
        matches!(&refused, Err(ConfigError::Invalid { key, .. }) if key == "switches.art_fallback.remove_after"),
        "{refused:?}"
    );
}

#[test]
fn storage_off_means_no_object_store() {
    assert_eq!(example().object_storage().unwrap(), None);
}

#[test]
fn a_missing_file_names_its_path() {
    let refused = Config::load(Some(std::path::Path::new("/nonexistent/myrtle.toml")));
    assert!(
        matches!(&refused, Err(ConfigError::File { path, .. }) if path == "/nonexistent/myrtle.toml"),
        "{refused:?}"
    );
}

fn file_secret(value: &str) -> (tempfile::NamedTempFile, SecretRef) {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    write!(file, "{value}").unwrap();
    let secret = SecretRef(format!("file:{}", file.path().display()));
    (file, secret)
}

#[test]
fn an_unset_optional_secret_does_not_stop_the_boot() {
    let mut config = example();
    let (_url, url) = file_secret("postgres://localhost/myrtle");
    let (_jwt, jwt) = file_secret("0123456789abcdef0123456789abcdef");
    let (_key, key) = file_secret(&"ab".repeat(32));
    config.database.url = url;
    config.secrets.jwt_secret = jwt;
    config.secrets.game_credential_key = key;
    let unset = || Some(SecretRef("env:MYRTLE_TEST_SECRET_THAT_IS_NOT_SET".into()));
    config.secrets.service_key = None;
    config.secrets.github_token = unset();
    config.secrets.discord_alert_webhook = unset();
    config.redis.url = unset();
    config.validate().unwrap();
    assert_eq!(SecretRef::optional(config.redis.url.as_ref()), None);

    config.secrets.jwt_secret = SecretRef("env:MYRTLE_TEST_SECRET_THAT_IS_NOT_SET".into());
    assert!(matches!(config.validate(), Err(ConfigError::Secret { .. })));
}

#[test]
fn zero_hash_threads_mean_the_storage_default() {
    let mut config = example();
    assert_eq!(
        config.storage.sync_hash_threads, 0,
        "the example ships zero"
    );
    config.storage.kind = "local".into();
    config.storage.local_dir = "/tmp/myrtle-objects".into();
    let storage = config.object_storage().unwrap().unwrap();
    assert!(storage.hash_threads >= 1);
    assert_eq!(storage.sync_concurrency, config.storage.sync_concurrency);
}

#[test]
fn storage_accepts_r2_and_local_only() {
    let mut config = example();
    for kind in ["aws", "s3"] {
        config.storage.kind = kind.into();
        assert!(matches!(
            config.object_storage(),
            Err(ConfigError::Invalid { key, .. }) if key == "storage"
        ));
    }
    config.storage.kind = "off".into();
    assert!(config.object_storage().unwrap().is_none());
}

#[test]
fn a_zero_dps_off_grid_ttl_reads_as_an_hour() {
    assert_eq!(example().dps.rank_off_grid_ttl_secs, 3600);
    let text = render_example();
    assert!(text.contains("rank_off_grid_ttl_secs = 3600"));
    let zero: Config = toml::from_str(&text.replace(
        "rank_off_grid_ttl_secs = 3600",
        "rank_off_grid_ttl_secs = 0",
    ))
    .unwrap();
    assert_eq!(zero.dps.rank_off_grid_ttl_secs, 3600);
    let day: Config = toml::from_str(&text.replace(
        "rank_off_grid_ttl_secs = 3600",
        "rank_off_grid_ttl_secs = 86400",
    ))
    .unwrap();
    assert_eq!(day.dps.rank_off_grid_ttl_secs, 86400);
}

#[test]
fn zero_sync_concurrency_fails_validation() {
    let mut config = example();
    config.storage.sync_concurrency = 0;
    let refused = config.validate();
    assert!(
        matches!(&refused, Err(ConfigError::Invalid { key, .. }) if key == "storage.sync_concurrency"),
        "{refused:?}"
    );
}

#[test]
fn an_unresolvable_storage_key_names_its_config_key() {
    let mut config = example();
    config.storage.kind = "cloudflare".into();
    config.storage.bucket = "assets".into();
    config.storage.account_id = "account".into();
    let (_secret, secret) = file_secret("secret-value");
    config.storage.secret_access_key = Some(secret);
    config.storage.access_key_id = Some(SecretRef("env:MYRTLE_TEST_SECRET_THAT_IS_NOT_SET".into()));
    let refused = config.object_storage();
    assert!(
        matches!(&refused, Err(ConfigError::Invalid { key, message })
            if key == "storage.access_key_id" && message.contains("MYRTLE_TEST_SECRET_THAT_IS_NOT_SET")),
        "{refused:?}"
    );

    config.storage.access_key_id = None;
    assert!(
        matches!(&config.object_storage(), Err(ConfigError::Invalid { key, message })
            if key == "storage" && message.contains("CLOUDFLARE_API_ACCESS_KEY")),
        "an absent key is still the storage crate's error"
    );
}

#[test]
fn an_unpadded_date_still_expires() {
    let mut config = example();
    config.switches.story_hub.remove_after = "2026-9-1".into();
    let expired: Vec<_> = config
        .expired_switches("2026-11-19")
        .into_iter()
        .map(|s| s.name)
        .collect();
    assert_eq!(expired, ["story_hub"]);
}
