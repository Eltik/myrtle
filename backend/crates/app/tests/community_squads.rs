#![allow(clippy::unwrap_used)]

mod common;

use myrtle_app::policy::Subject;
use myrtle_app::services::community::squads::{
    self, SavedSquadInput, SquadBuild, SquadMember, check_member_build, checked, checked_slot,
    range, seats,
};
use myrtle_app::snapshots::Snapshot;
use myrtle_app::{AppContext, AppError};
use myrtle_engine_squad::SquadAnalyzeRequest;
use myrtle_engine_squad::policy::SQUAD_SIZE;
use myrtle_engine_squad::types::SquadMemberBuild;
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::sandbox_universe::SandboxUniverse;
use myrtle_types::{Role, Server};
use serde_json::json;

fn build(value: i32) -> SquadMemberBuild {
    SquadMemberBuild {
        char_id: "char_002_amiya".into(),
        phase: value,
        level: value,
        potential: value,
        skill_index: value,
        skill_level: value,
        mastery: value,
        module_id: None,
        module_level: value,
    }
}

fn member(char_id: &str) -> SquadMember {
    SquadMember {
        char_id: char_id.into(),
        skill_index: 0,
        module_id: None,
        build: None,
    }
}

fn input(members: Vec<Option<SquadMember>>, stage_id: Option<&str>) -> SavedSquadInput {
    SavedSquadInput {
        name: "  ".into(),
        members,
        support: None,
        stage_id: stage_id.map(str::to_string),
    }
}

fn message<T: std::fmt::Debug>(r: Result<T, AppError>) -> String {
    match r {
        Err(AppError::BadRequest(m)) => m,
        other => panic!("expected a 400, got {other:?}"),
    }
}

fn install_empty_data(ctx: &AppContext) {
    let snapshot = Snapshot::from_model(GameModel::default(), SandboxUniverse::default());
    ctx.snapshots
        .install(ctx.snapshots.default_server(), snapshot);
}

const fn analyze_request(members: Vec<Option<SquadMemberBuild>>) -> SquadAnalyzeRequest {
    SquadAnalyzeRequest {
        members,
        support: None,
        start_dp: 10,
        deploy_limit: Some(8),
        dub: None,
    }
}

#[test]
fn slot_and_range_messages() {
    assert_eq!(checked_slot(1).unwrap(), 1);
    assert_eq!(checked_slot(4).unwrap(), 4);
    assert_eq!(message(checked_slot(0)), "slot must be 1 to 4, got 0");
    assert_eq!(message(checked_slot(5)), "slot must be 1 to 4, got 5");
    assert!(range("x", 3, 0, 3).is_ok());
    assert_eq!(message(range("x", -1, 0, 3)), "x must be 0 to 3, got -1");
}

#[test]
fn analyze_build_caps() {
    assert!(check_member_build("members[0]", &build(999)).is_ok());
    assert_eq!(
        message(check_member_build("members[0]", &build(1000))),
        "members[0].phase must be 0 to 999, got 1000"
    );
    let mut b = build(1);
    b.mastery = -1;
    assert_eq!(
        message(check_member_build("support", &b)),
        "support.mastery must be 0 to 999, got -1"
    );
}

#[test]
fn saved_squad_checks() {
    let data = GameModel::default();
    let ok = checked(&data, 2, input(Vec::new(), Some(" main_01-07 "))).unwrap();
    assert_eq!(ok.name, "Squad 2");
    assert_eq!(ok.members.len(), SQUAD_SIZE);
    assert_eq!(ok.stage_id.as_deref(), Some("main_01-07"));
    assert_eq!(
        checked(&data, 1, input(Vec::new(), Some("   ")))
            .unwrap()
            .stage_id,
        None
    );
    assert_eq!(
        message(checked(&data, 1, input(vec![None; 13], None))),
        "members: at most 12 seats, got 13"
    );
    assert_eq!(
        message(checked(
            &data,
            1,
            input(vec![None, Some(member("char_x"))], None)
        )),
        "members[1]: unknown operator `char_x`"
    );
    let mut with_support = input(Vec::new(), None);
    with_support.support = Some(member("char_y"));
    assert_eq!(
        message(checked(&data, 1, with_support)),
        "support: unknown operator `char_y`"
    );
    assert_eq!(
        message(checked(&data, 1, input(Vec::new(), Some("a b")))),
        "stage_id may hold only letters, digits, `_`, `#` and `-`"
    );
    let mut long_name = input(Vec::new(), None);
    long_name.name = "x".repeat(41);
    assert_eq!(message(checked(&data, 1, long_name)), "name max 40 chars");
}

#[test]
fn stored_seats_are_twelve_and_drop_what_does_not_parse() {
    let parsed: Vec<Option<SquadMember>> = seats(json!([
        {"char_id": "char_002_amiya", "skill_index": 1, "module_id": null, "build": null},
        {"char_id": 5},
        null
    ]));
    assert_eq!(parsed.len(), SQUAD_SIZE);
    assert_eq!(parsed[0].as_ref().unwrap().skill_index, 1);
    assert!(parsed[1..].iter().all(Option::is_none));
    let long: Vec<Option<SquadMember>> = seats(json!(vec![serde_json::Value::Null; 20]));
    assert_eq!(long.len(), SQUAD_SIZE);
    assert_eq!(
        seats::<SquadMember>(json!({"not": "an array"})).len(),
        SQUAD_SIZE
    );
}

#[tokio::test]
async fn saved_squads_round_trip_per_account() {
    let (ctx, db) = common::context().await;
    install_empty_data(&ctx);
    let me = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let other = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;

    let empty = squads::get(&ctx, &me).await.unwrap();
    assert_eq!(empty.saved.len(), 4);
    assert!(empty.saved.iter().all(Option::is_none));
    assert!(empty.synced.is_empty());
    assert_eq!((empty.slot_limit, empty.squad_size), (4, 12));

    let mut named = input(Vec::new(), Some("main_01-07"));
    named.name = " Night shift ".into();
    let saved = squads::save(&ctx, &me, 2, named, Server::EN).await.unwrap();
    assert_eq!(saved.slot, 2);
    assert_eq!(saved.name, "Night shift");
    assert_eq!(saved.members.len(), SQUAD_SIZE);

    let replaced = squads::save(&ctx, &me, 2, input(Vec::new(), None), Server::EN)
        .await
        .unwrap();
    assert_eq!(replaced.name, "Squad 2");
    assert_eq!(replaced.stage_id, None);

    let read = squads::get(&ctx, &me).await.unwrap();
    assert!(read.saved[0].is_none());
    assert_eq!(read.saved[1].as_ref().unwrap().name, "Squad 2");
    assert!(
        squads::get(&ctx, &other)
            .await
            .unwrap()
            .saved
            .iter()
            .all(Option::is_none)
    );

    assert!(matches!(
        squads::delete(&ctx, &other, 2).await,
        Err(AppError::NotFound(None))
    ));
    squads::delete(&ctx, &me, 2).await.unwrap();
    assert!(matches!(
        squads::delete(&ctx, &me, 2).await,
        Err(AppError::NotFound(None))
    ));
    db.close().await;
}

#[tokio::test]
async fn squads_refuse_anonymous_callers_and_bad_input() {
    let (ctx, db) = common::context().await;
    install_empty_data(&ctx);
    let me = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;

    assert!(matches!(
        squads::get(&ctx, &Subject::Anonymous).await,
        Err(AppError::Unauthorized)
    ));
    assert!(matches!(
        squads::save(
            &ctx,
            &Subject::Anonymous,
            1,
            input(Vec::new(), None),
            Server::EN
        )
        .await,
        Err(AppError::Unauthorized)
    ));
    assert!(matches!(
        squads::delete(&ctx, &Subject::Anonymous, 1).await,
        Err(AppError::Unauthorized)
    ));
    assert_eq!(
        message(squads::save(&ctx, &me, 9, input(Vec::new(), None), Server::EN).await),
        "slot must be 1 to 4, got 9"
    );
    assert_eq!(
        message(
            squads::save(
                &ctx,
                &me,
                1,
                input(vec![Some(member("char_x"))], None),
                Server::EN
            )
            .await
        ),
        "members[0]: unknown operator `char_x`"
    );
    assert!(squads::get(&ctx, &me).await.unwrap().saved[0].is_none());
    db.close().await;
}

#[tokio::test]
async fn analyze_checks_the_request_before_running() {
    let (ctx, db) = common::context().await;
    install_empty_data(&ctx);
    let mut request = analyze_request(Vec::new());
    request.start_dp = 1000;
    assert_eq!(
        message(squads::analyze(&ctx, request, Server::EN).await),
        "start_dp must be 0 to 999, got 1000"
    );
    let mut request = analyze_request(Vec::new());
    request.deploy_limit = Some(21);
    assert_eq!(
        message(squads::analyze(&ctx, request, Server::EN).await),
        "deploy_limit must be 0 to 20, got 21"
    );
    assert_eq!(
        message(squads::analyze(&ctx, analyze_request(vec![None; 13]), Server::EN).await),
        "members: at most 12 seats, got 13"
    );
    let empty = squads::analyze(&ctx, analyze_request(Vec::new()), Server::EN)
        .await
        .unwrap();
    assert!(empty.unread_seats.is_empty());
    db.close().await;
}

#[tokio::test]
async fn squads_check_and_analyze_against_real_data() {
    let Some((ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let me = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;

    let mut amiya = member("char_002_amiya");
    amiya.build = Some(SquadBuild {
        phase: 2,
        level: 1,
        potential: 0,
        skill_level: 7,
        mastery: 3,
        module_level: 0,
    });
    let saved = squads::save(
        &ctx,
        &me,
        1,
        input(vec![Some(amiya.clone())], None),
        Server::EN,
    )
    .await
    .unwrap();
    assert_eq!(saved.members[0].as_ref(), Some(&amiya));

    let mut too_high = amiya.clone();
    too_high.build.as_mut().unwrap().level = 1000;
    assert!(
        message(squads::save(&ctx, &me, 1, input(vec![Some(too_high)], None), Server::EN).await)
            .starts_with("members[0].build.level must be 1 to ")
    );
    let mut bad_skill = amiya;
    bad_skill.skill_index = 9;
    assert_eq!(
        message(squads::save(&ctx, &me, 1, input(vec![Some(bad_skill)], None), Server::EN).await),
        "members[0]: skill_index 9 is not one of `char_002_amiya`'s skills"
    );

    let rules = squads::rules(&ctx, Server::EN).unwrap();
    assert_eq!(rules.normal_of, snapshot.model.squad.normal_of);

    let mut seat = build(1);
    seat.phase = 2;
    seat.level = 90;
    seat.skill_index = 0;
    seat.skill_level = 7;
    seat.mastery = 3;
    let analysis = squads::analyze(&ctx, analyze_request(vec![Some(seat)]), Server::EN)
        .await
        .unwrap();
    assert!(analysis.unread_seats.is_empty());
    assert!(!analysis.handoff.is_empty());
    db.close().await;
}
