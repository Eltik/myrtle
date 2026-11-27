#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::Conn;
use myrtle_db::repos::oauth;
use myrtle_types::UserId;
use sqlx::PgPool;
use uuid::Uuid;

const HOUR: i64 = 3600;

async fn register(conn: &mut Conn, owner: UserId, name: &str) -> oauth::AppRow {
    oauth::insert_app(
        conn,
        owner,
        name,
        "",
        &["https://client.example/cb".to_owned()],
        format!("secret-{name}").as_bytes(),
    )
    .await
    .unwrap()
}

/// A grant of `app` to `player` holding one session with tokens `a-<tag>` and `r-<tag>`
async fn signed_in(conn: &mut Conn, app: Uuid, player: UserId, tag: &str) -> Uuid {
    let grant = oauth::upsert_grant(conn, app, player, "identify")
        .await
        .unwrap();
    session(conn, grant, tag).await;
    grant
}

async fn session(conn: &mut Conn, grant: Uuid, tag: &str) {
    oauth::insert_token(
        conn,
        grant,
        format!("a-{tag}").as_bytes(),
        format!("r-{tag}").as_bytes(),
        "identify",
        HOUR,
        HOUR,
        5,
    )
    .await
    .unwrap();
}

fn names(rows: &[oauth::AuthorizationRow]) -> Vec<&str> {
    rows.iter().map(|r| r.app.name.as_str()).collect()
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn an_owner_lists_their_apps_in_registration_order_with_live_grant_counts(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;
    let player = common::user(&mut conn, 3).await;

    let first = register(&mut conn, owner, "first").await;
    assert_eq!(first.authorized_users, 0);
    assert_eq!(first.redirect_uris, ["https://client.example/cb"]);
    let second = register(&mut conn, owner, "second").await;
    register(&mut conn, other, "theirs").await;
    oauth::upsert_grant(&mut conn, second.id, player, "identify")
        .await
        .unwrap();

    let apps = oauth::list_apps(&mut conn, owner).await.unwrap();
    let listed: Vec<(&str, i64)> = apps
        .iter()
        .map(|a| (a.name.as_str(), a.authorized_users))
        .collect();
    assert_eq!(listed, [("first", 0), ("second", 1)]);
    assert_eq!(oauth::count_apps(&mut conn, owner).await.unwrap(), 2);
    assert_eq!(oauth::count_apps(&mut conn, player).await.unwrap(), 0);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn only_the_owner_edits_an_app_or_replaces_its_secret(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;
    let app = register(&mut conn, owner, "app").await;
    let uris = ["https://new.example/cb".to_owned()];

    let refused = oauth::update_app(&mut conn, app.id, other, "x", "y", &uris)
        .await
        .unwrap();
    assert!(refused.is_none());
    let updated = oauth::update_app(&mut conn, app.id, owner, "Renamed", "about", &uris)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.name, "Renamed");
    assert_eq!(updated.description, "about");
    assert_eq!(updated.redirect_uris, uris);
    assert_eq!(updated.created_at, app.created_at);

    assert!(
        oauth::replace_secret(&mut conn, app.id, other, b"stolen")
            .await
            .unwrap()
            .is_none()
    );
    oauth::replace_secret(&mut conn, app.id, owner, b"fresh")
        .await
        .unwrap()
        .unwrap();
    let hash = oauth::find_secret_hash(&mut conn, app.id).await.unwrap();
    assert_eq!(hash.as_deref(), Some(&b"fresh"[..]));
    assert!(
        oauth::find_secret_hash(&mut conn, Uuid::new_v4())
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn deleting_an_app_is_its_owners_unless_unscoped_and_takes_every_session(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;
    let player = common::user(&mut conn, 3).await;
    let app = register(&mut conn, owner, "app").await;
    let taken_down = register(&mut conn, owner, "taken down").await;
    signed_in(&mut conn, app.id, player, "1").await;

    assert!(
        !oauth::delete_app(&mut conn, app.id, Some(other))
            .await
            .unwrap()
    );
    assert!(
        oauth::delete_app(&mut conn, app.id, Some(owner))
            .await
            .unwrap()
    );
    assert!(!oauth::has_grant(&mut conn, app.id, player).await.unwrap());
    assert!(
        oauth::find_access_token(&mut conn, b"a-1")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        oauth::delete_app(&mut conn, taken_down.id, None)
            .await
            .unwrap()
    );
    assert!(
        !oauth::delete_app(&mut conn, taken_down.id, None)
            .await
            .unwrap()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_public_app_names_the_account_that_registered_it(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    sqlx::query("UPDATE users SET nick_number = '1234' WHERE id = $1")
        .bind(owner.0)
        .execute(&mut *conn)
        .await
        .unwrap();
    let app = register(&mut conn, owner, "app").await;

    let public = oauth::find_public_app(&mut conn, app.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(public.name, "app");
    assert_eq!(public.owner_nickname.as_deref(), Some("user1"));
    assert_eq!(public.owner_nick_number.as_deref(), Some("1234"));
    assert_eq!(public.redirect_uris, app.redirect_uris);
    assert!(
        oauth::find_public_app(&mut conn, Uuid::new_v4())
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn approving_again_keeps_the_grant_and_replaces_its_scope(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let player = common::user(&mut conn, 2).await;
    let app = register(&mut conn, owner, "app").await;
    assert!(!oauth::has_grant(&mut conn, app.id, player).await.unwrap());

    let first = oauth::upsert_grant(&mut conn, app.id, player, "identify")
        .await
        .unwrap();
    let created: chrono::DateTime<chrono::Utc> =
        sqlx::query_scalar("SELECT created_at FROM oauth_grants WHERE id = $1")
            .bind(first)
            .fetch_one(&mut *conn)
            .await
            .unwrap();
    let again = oauth::upsert_grant(&mut conn, app.id, player, "identify email")
        .await
        .unwrap();
    assert_eq!(again, first);
    let (scope, created_again): (String, chrono::DateTime<chrono::Utc>) =
        sqlx::query_as("SELECT scope, created_at FROM oauth_grants WHERE id = $1")
            .bind(first)
            .fetch_one(&mut *conn)
            .await
            .unwrap();
    assert_eq!(scope, "identify email");
    assert_eq!(created_again, created);
    assert!(oauth::has_grant(&mut conn, app.id, player).await.unwrap());
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_code_is_spent_once_and_only_by_its_own_application(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let player = common::user(&mut conn, 2).await;
    let app = register(&mut conn, owner, "app").await;
    let rival = register(&mut conn, owner, "rival").await;
    let grant = oauth::upsert_grant(&mut conn, app.id, player, "identify")
        .await
        .unwrap();
    oauth::insert_code(
        &mut conn,
        b"code",
        grant,
        "https://client.example/cb",
        "identify",
        Some("challenge"),
        300,
    )
    .await
    .unwrap();

    assert!(
        oauth::consume_code(&mut conn, b"code", rival.id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        oauth::find_spent_code_grant(&mut conn, b"code", app.id)
            .await
            .unwrap()
            .is_none()
    );
    let spent = oauth::consume_code(&mut conn, b"code", app.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(spent.grant_id, grant);
    assert_eq!(spent.redirect_uri, "https://client.example/cb");
    assert_eq!(spent.scope, "identify");
    assert_eq!(spent.code_challenge.as_deref(), Some("challenge"));
    assert!(
        oauth::consume_code(&mut conn, b"code", app.id)
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        oauth::find_spent_code_grant(&mut conn, b"code", app.id)
            .await
            .unwrap(),
        Some(grant)
    );
    assert!(
        oauth::find_spent_code_grant(&mut conn, b"code", rival.id)
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn an_expired_code_cannot_be_spent(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let player = common::user(&mut conn, 2).await;
    let app = register(&mut conn, owner, "app").await;
    let grant = oauth::upsert_grant(&mut conn, app.id, player, "identify")
        .await
        .unwrap();
    oauth::insert_code(
        &mut conn,
        b"old",
        grant,
        "https://client.example/cb",
        "identify",
        None,
        -1,
    )
    .await
    .unwrap();

    assert!(
        oauth::consume_code(&mut conn, b"old", app.id)
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_replayed_code_signs_the_grant_out_and_keeps_the_consent(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let player = common::user(&mut conn, 2).await;
    let app = register(&mut conn, owner, "app").await;
    let grant = signed_in(&mut conn, app.id, player, "1").await;
    session(&mut conn, grant, "2").await;

    assert_eq!(
        oauth::delete_grant_tokens(&mut conn, grant).await.unwrap(),
        2
    );
    assert!(
        oauth::find_access_token(&mut conn, b"a-2")
            .await
            .unwrap()
            .is_none()
    );
    assert!(oauth::has_grant(&mut conn, app.id, player).await.unwrap());
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_new_session_stamps_the_grant_and_drops_the_oldest_past_keep(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let player = common::user(&mut conn, 2).await;
    let app = register(&mut conn, owner, "app").await;
    let grant = signed_in(&mut conn, app.id, player, "old").await;
    session(&mut conn, grant, "mid").await;
    sqlx::query(
        "UPDATE oauth_tokens SET created_at = created_at - interval '1 hour'
         WHERE access_hash = 'a-old'::bytea",
    )
    .execute(&mut *conn)
    .await
    .unwrap();
    let last_used: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT last_used_at FROM oauth_grants WHERE id = $1")
            .bind(grant)
            .fetch_one(&mut *conn)
            .await
            .unwrap();
    assert!(last_used.is_some());

    oauth::insert_token(
        &mut conn, grant, b"a-new", b"r-new", "identify", HOUR, HOUR, 2,
    )
    .await
    .unwrap();

    assert!(
        oauth::find_access_token(&mut conn, b"a-old")
            .await
            .unwrap()
            .is_none()
    );
    for live in [&b"a-mid"[..], &b"a-new"[..]] {
        let owner_row = oauth::find_access_token(&mut conn, live)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(owner_row.user_id, player.0);
        assert_eq!(owner_row.scope, "identify");
    }
}

/// Makes every later stamp of a grant fail
async fn break_grant_stamps(conn: &mut Conn) {
    sqlx::raw_sql(
        "CREATE FUNCTION refuse() RETURNS trigger LANGUAGE plpgsql
             AS $$ BEGIN RAISE EXCEPTION 'refused'; END $$;
         CREATE TRIGGER refuse BEFORE UPDATE ON oauth_grants
             FOR EACH ROW EXECUTE FUNCTION refuse();",
    )
    .execute(conn)
    .await
    .unwrap();
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_session_whose_grant_stamp_fails_is_not_kept_and_prunes_nothing(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let player = common::user(&mut conn, 2).await;
    let app = register(&mut conn, owner, "app").await;
    let grant = signed_in(&mut conn, app.id, player, "old").await;
    break_grant_stamps(&mut conn).await;

    let failed = oauth::insert_token(
        &mut conn, grant, b"a-new", b"r-new", "identify", HOUR, HOUR, 1,
    )
    .await;
    assert!(failed.is_err());
    assert!(
        oauth::find_access_token(&mut conn, b"a-new")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        oauth::find_access_token(&mut conn, b"a-old")
            .await
            .unwrap()
            .is_some()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_rotation_whose_grant_stamp_fails_keeps_the_old_tokens(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let player = common::user(&mut conn, 2).await;
    let app = register(&mut conn, owner, "app").await;
    signed_in(&mut conn, app.id, player, "1").await;
    break_grant_stamps(&mut conn).await;

    let failed = oauth::rotate_token(&mut conn, b"r-1", app.id, b"a-2", b"r-2", HOUR, HOUR).await;
    assert!(failed.is_err());
    assert!(
        oauth::find_access_token(&mut conn, b"a-1")
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        oauth::find_access_token(&mut conn, b"a-2")
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn an_expired_access_token_speaks_for_no_one(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let player = common::user(&mut conn, 2).await;
    let app = register(&mut conn, owner, "app").await;
    let grant = oauth::upsert_grant(&mut conn, app.id, player, "identify")
        .await
        .unwrap();
    oauth::insert_token(&mut conn, grant, b"a", b"r", "identify", -1, HOUR, 5)
        .await
        .unwrap();

    assert!(
        oauth::find_access_token(&mut conn, b"a")
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_refresh_token_rotates_once_and_only_for_its_application(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let player = common::user(&mut conn, 2).await;
    let app = register(&mut conn, owner, "app").await;
    let rival = register(&mut conn, owner, "rival").await;
    signed_in(&mut conn, app.id, player, "1").await;

    let refused = oauth::rotate_token(&mut conn, b"r-1", rival.id, b"x", b"y", HOUR, HOUR)
        .await
        .unwrap();
    assert!(refused.is_none());
    let scope = oauth::rotate_token(&mut conn, b"r-1", app.id, b"a-2", b"r-2", HOUR, HOUR)
        .await
        .unwrap();
    assert_eq!(scope.as_deref(), Some("identify"));
    let replay = oauth::rotate_token(&mut conn, b"r-1", app.id, b"a-3", b"r-3", HOUR, HOUR)
        .await
        .unwrap();
    assert!(replay.is_none());
    assert!(
        oauth::find_access_token(&mut conn, b"a-1")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        oauth::find_access_token(&mut conn, b"a-2")
            .await
            .unwrap()
            .is_some()
    );
    let sessions: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM oauth_tokens")
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    assert_eq!(sessions, 1);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn an_expired_refresh_token_does_not_rotate(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let player = common::user(&mut conn, 2).await;
    let app = register(&mut conn, owner, "app").await;
    let grant = oauth::upsert_grant(&mut conn, app.id, player, "identify")
        .await
        .unwrap();
    oauth::insert_token(&mut conn, grant, b"a", b"r", "identify", HOUR, -1, 5)
        .await
        .unwrap();

    let rotated = oauth::rotate_token(&mut conn, b"r", app.id, b"a2", b"r2", HOUR, HOUR)
        .await
        .unwrap();
    assert!(rotated.is_none());
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn revoking_either_token_ends_the_session_for_the_issuing_app_only(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let player = common::user(&mut conn, 2).await;
    let app = register(&mut conn, owner, "app").await;
    let rival = register(&mut conn, owner, "rival").await;
    signed_in(&mut conn, app.id, player, "1").await;
    let grant = oauth::upsert_grant(&mut conn, app.id, player, "identify")
        .await
        .unwrap();
    session(&mut conn, grant, "2").await;

    assert!(
        !oauth::revoke_token(&mut conn, b"a-1", rival.id)
            .await
            .unwrap()
    );
    assert!(
        oauth::revoke_token(&mut conn, b"a-1", app.id)
            .await
            .unwrap()
    );
    assert!(
        !oauth::revoke_token(&mut conn, b"r-1", app.id)
            .await
            .unwrap()
    );
    assert!(
        oauth::revoke_token(&mut conn, b"r-2", app.id)
            .await
            .unwrap()
    );
    assert!(
        oauth::find_access_token(&mut conn, b"a-2")
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_identity_carries_the_server_code(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let player = common::user(&mut conn, 7).await;
    sqlx::query("UPDATE users SET avatar_id = 'char_002_amiya' WHERE id = $1")
        .bind(player.0)
        .execute(&mut *conn)
        .await
        .unwrap();

    let me = oauth::find_identity(&mut conn, player)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(me.id, player.0);
    assert_eq!(me.uid, "7");
    assert_eq!(me.server, "en");
    assert_eq!(me.nickname.as_deref(), Some("user7"));
    assert_eq!(me.nick_number, None);
    assert_eq!(me.avatar_id.as_deref(), Some("char_002_amiya"));
    assert!(
        oauth::find_identity(&mut conn, UserId(Uuid::new_v4()))
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn authorizations_list_most_recently_used_first_and_withdrawing_ends_sessions(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let player = common::user(&mut conn, 2).await;
    let used = register(&mut conn, owner, "used").await;
    let fresh = register(&mut conn, owner, "fresh").await;
    let grant = oauth::upsert_grant(&mut conn, used.id, player, "identify")
        .await
        .unwrap();
    oauth::upsert_grant(&mut conn, fresh.id, player, "identify")
        .await
        .unwrap();

    let before = oauth::list_authorizations(&mut conn, player).await.unwrap();
    assert_eq!(names(&before), ["fresh", "used"]);
    assert!(before.iter().all(|r| r.last_used_at.is_none()));

    session(&mut conn, grant, "1").await;
    let after = oauth::list_authorizations(&mut conn, player).await.unwrap();
    assert_eq!(names(&after), ["used", "fresh"]);
    assert!(after[0].last_used_at.is_some());
    assert_eq!(after[0].scope, "identify");
    assert_eq!(after[0].app.owner_nickname.as_deref(), Some("user1"));
    assert!(
        oauth::list_authorizations(&mut conn, owner)
            .await
            .unwrap()
            .is_empty()
    );

    assert!(
        oauth::delete_grant(&mut conn, used.id, player)
            .await
            .unwrap()
    );
    assert!(
        !oauth::delete_grant(&mut conn, used.id, player)
            .await
            .unwrap()
    );
    assert!(
        oauth::find_access_token(&mut conn, b"a-1")
            .await
            .unwrap()
            .is_none()
    );
    let left = oauth::list_authorizations(&mut conn, player).await.unwrap();
    assert_eq!(names(&left), ["fresh"]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn sweeping_drops_expired_codes_and_sessions_whose_refresh_ran_out(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let player = common::user(&mut conn, 2).await;
    let app = register(&mut conn, owner, "app").await;
    let grant = oauth::upsert_grant(&mut conn, app.id, player, "identify")
        .await
        .unwrap();
    let uri = "https://client.example/cb";
    oauth::insert_code(&mut conn, b"dead", grant, uri, "identify", None, -1)
        .await
        .unwrap();
    oauth::insert_code(&mut conn, b"live", grant, uri, "identify", None, 300)
        .await
        .unwrap();
    oauth::insert_token(
        &mut conn, grant, b"a-dead", b"r-dead", "identify", -1, -1, 5,
    )
    .await
    .unwrap();
    oauth::insert_token(
        &mut conn, grant, b"a-stale", b"r-live", "identify", -1, HOUR, 5,
    )
    .await
    .unwrap();

    assert_eq!(oauth::delete_expired(&mut conn).await.unwrap(), 2);
    assert!(
        oauth::consume_code(&mut conn, b"live", app.id)
            .await
            .unwrap()
            .is_some()
    );
    assert_eq!(
        oauth::rotate_token(&mut conn, b"r-live", app.id, b"a", b"r", HOUR, HOUR)
            .await
            .unwrap()
            .as_deref(),
        Some("identify")
    );
    assert_eq!(oauth::delete_expired(&mut conn).await.unwrap(), 0);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn deleting_an_account_ends_what_it_authorized_and_what_it_registered(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let developer = common::user(&mut conn, 1).await;
    let player = common::user(&mut conn, 2).await;
    let other = common::user(&mut conn, 3).await;
    let theirs = register(&mut conn, other, "theirs").await;
    let mine = register(&mut conn, developer, "mine").await;
    signed_in(&mut conn, theirs.id, player, "p").await;
    signed_in(&mut conn, mine.id, other, "o").await;

    for gone in [developer, player] {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(gone.0)
            .execute(&mut *conn)
            .await
            .unwrap();
    }

    assert!(
        oauth::find_public_app(&mut conn, mine.id)
            .await
            .unwrap()
            .is_none()
    );
    for token in [&b"a-p"[..], &b"a-o"[..]] {
        assert!(
            oauth::find_access_token(&mut conn, token)
                .await
                .unwrap()
                .is_none()
        );
    }
    let apps = oauth::list_apps(&mut conn, other).await.unwrap();
    assert_eq!(apps[0].authorized_users, 0);
}
