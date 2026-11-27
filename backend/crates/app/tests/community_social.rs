#![allow(clippy::unwrap_used)]

mod common;

use myrtle_app::AppError;
use myrtle_app::policy::Subject;
use myrtle_app::services::community::social::{
    self, friend_info_request, friends_request, listed_uids, no_friends, search_request,
    upstream_error,
};
use myrtle_hypergryph::UpstreamError;
use myrtle_types::{Role, Server};
use serde_json::json;

#[test]
fn sort_list_requests_match_the_game() {
    assert_eq!(
        friends_request(),
        json!({"type": 1, "sortKeyList": ["level", "infoShare"], "param": {}})
    );
    assert_eq!(
        search_request("Doctor#1234"),
        json!({
            "type": 0,
            "sortKeyList": ["level"],
            "param": {"nickName": "Doctor", "nickNumber": "1234"}
        })
    );
    assert_eq!(
        search_request("Doctor")["param"],
        json!({"nickName": "Doctor", "nickNumber": ""})
    );
    assert_eq!(
        friend_info_request(&["1".into(), "2".into()]),
        json!({"idList": ["1", "2"]})
    );
    assert_eq!(no_friends(), json!({"friends": []}));
}

#[test]
fn listed_uids_keep_order_skip_entries_without_one_and_honour_the_limit() {
    let raw = json!({
        "result": [
            {"uid": "11", "level": 120},
            {"level": 3},
            {"uid": 7},
            {"uid": "22"},
            {"uid": "33"}
        ],
        "playerDataDelta": {}
    });
    assert_eq!(listed_uids(&raw, None).unwrap(), ["11", "22", "33"]);
    assert_eq!(listed_uids(&raw, Some(2)).unwrap(), ["11", "22"]);
    assert!(listed_uids(&raw, Some(0)).unwrap().is_empty());
    assert!(
        listed_uids(&json!({"result": []}), None)
            .unwrap()
            .is_empty()
    );
    for malformed in [json!({}), json!({"result": {"uid": "1"}})] {
        assert!(matches!(
            listed_uids(&malformed, None),
            Err(AppError::BadRequest(m)) if m == "malformed social response"
        ));
    }
}

#[test]
fn a_missing_game_session_asks_for_a_login() {
    assert!(matches!(
        upstream_error(UpstreamError::NotLoggedIn),
        AppError::GameLoginRequired
    ));
    assert!(matches!(
        upstream_error(UpstreamError::Unsupported {
            server: Server::CN,
            what: "social"
        }),
        AppError::Upstream(UpstreamError::Unsupported { .. })
    ));
}

#[tokio::test]
async fn lookups_need_an_account_with_a_game_session() {
    let (ctx, db) = common::context().await;
    assert!(matches!(
        social::friends(&ctx, &Subject::Anonymous, None).await,
        Err(AppError::Unauthorized)
    ));
    let me = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    assert!(matches!(
        social::friends(&ctx, &me, Some(5)).await,
        Err(AppError::GameLoginRequired)
    ));
    assert!(matches!(
        social::search_players(&ctx, &me, "Doctor#1234", None).await,
        Err(AppError::GameLoginRequired)
    ));
    db.close().await;
}
