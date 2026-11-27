#![allow(clippy::unwrap_used)]

mod common;

use myrtle_app::policy::Subject;
use myrtle_app::services::oauth::apps;
use myrtle_app::services::oauth::dto::OAuthAppInput;
use myrtle_app::services::oauth::protocol::{
    MAX_DESCRIPTION_LEN, MAX_NAME_LEN, MAX_REDIRECT_URIS, digest,
};
use myrtle_app::{AppContext, AppError};
use myrtle_db::repos::oauth as repo;
use myrtle_types::{Role, Server};

fn input(name: &str, description: Option<&str>, uris: &[&str]) -> OAuthAppInput {
    OAuthAppInput {
        name: name.to_owned(),
        description: description.map(str::to_owned),
        redirect_uris: uris.iter().map(|u| (*u).to_owned()).collect(),
    }
}

async fn refused_fields(ctx: &AppContext, owner: &Subject, input: OAuthAppInput) -> Vec<String> {
    match apps::create(ctx, owner, input).await {
        Err(AppError::Validation(errors)) => errors.into_iter().map(|e| e.field).collect(),
        other => panic!("expected a validation failure, got {other:?}"),
    }
}

#[tokio::test]
async fn a_valid_application_is_trimmed_its_uris_deduplicated_and_its_secret_hashed() {
    let (ctx, db) = common::context().await;
    let owner = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;

    let created = apps::create(
        &ctx,
        &owner,
        input(
            "  miowawa.de ",
            Some(" connections "),
            &[
                "https://miowawa.de/cb",
                "http://localhost:4000/cb",
                "https://miowawa.de/cb",
            ],
        ),
    )
    .await
    .unwrap();
    assert_eq!(created.app.name, "miowawa.de");
    assert_eq!(created.app.description, "connections");
    assert_eq!(
        created.app.redirect_uris,
        ["https://miowawa.de/cb", "http://localhost:4000/cb"]
    );
    let stored = repo::find_secret_hash(&mut ctx.db.conn().await.unwrap(), created.app.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        stored,
        digest(&created.client_secret),
        "only the hash is kept"
    );
    db.close().await;
}

#[tokio::test]
async fn every_bad_field_is_reported_in_one_response() {
    let (ctx, db) = common::context().await;
    let owner = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let long_name = "n".repeat(MAX_NAME_LEN + 1);
    let long_description = "d".repeat(MAX_DESCRIPTION_LEN + 1);

    assert_eq!(
        refused_fields(
            &ctx,
            &owner,
            input(
                &long_name,
                Some(&long_description),
                &["http://example.com/cb"]
            )
        )
        .await,
        ["name", "description", "redirect_uris"]
    );
    assert_eq!(
        refused_fields(&ctx, &owner, input("   ", None, &[])).await,
        ["name", "redirect_uris"]
    );
    assert_eq!(
        refused_fields(
            &ctx,
            &owner,
            input("a\nb", Some("x\ny"), &["https://a.example/cb"])
        )
        .await,
        ["name", "description"]
    );
    db.close().await;
}

#[tokio::test]
async fn an_application_takes_at_most_ten_redirect_uris() {
    let (ctx, db) = common::context().await;
    let owner = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let uris: Vec<String> = (0..=MAX_REDIRECT_URIS)
        .map(|i| format!("https://example.com/cb/{i}"))
        .collect();
    let refs: Vec<&str> = uris.iter().map(String::as_str).collect();

    assert_eq!(
        refused_fields(&ctx, &owner, input("app", None, &refs)).await,
        ["redirect_uris"]
    );
    apps::create(&ctx, &owner, input("app", None, &refs[..MAX_REDIRECT_URIS]))
        .await
        .unwrap();
    db.close().await;
}

#[tokio::test]
async fn an_oversized_redirect_list_is_refused_without_comparing_every_pair() {
    let (ctx, db) = common::context().await;
    let owner = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let uris: Vec<String> = (0..200_000)
        .map(|i| format!("https://example.com/cb/{i}"))
        .collect();
    let refs: Vec<&str> = uris.iter().map(String::as_str).collect();

    let started = std::time::Instant::now();
    assert_eq!(
        refused_fields(&ctx, &owner, input("app", None, &refs)).await,
        ["redirect_uris"]
    );
    assert!(started.elapsed().as_secs() < 10, "{:?}", started.elapsed());
    db.close().await;
}

#[tokio::test]
async fn only_an_account_manages_applications() {
    let (ctx, db) = common::context().await;
    let app = input("app", None, &["https://example.com/cb"]);
    for subject in [Subject::Anonymous, Subject::System] {
        assert!(
            matches!(
                apps::create(&ctx, &subject, app.clone()).await,
                Err(AppError::Unauthorized)
            ),
            "{subject:?}"
        );
    }
    let ghost = common::user(&ctx, myrtle_types::UserId(uuid::Uuid::new_v4()), Role::User).await;
    assert!(
        matches!(
            apps::create(&ctx, &ghost, app).await,
            Err(AppError::Unauthorized)
        ),
        "a token that outlived its account"
    );
    db.close().await;
}
