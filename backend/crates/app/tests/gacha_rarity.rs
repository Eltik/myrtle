#![allow(clippy::unwrap_used)]

mod common;

use std::collections::HashMap;

use myrtle_app::services::gacha::rarity::{RarityFix, plan_against, rarity_fixes};
use myrtle_db::repos::gacha::{self, CharRarityRow};
use myrtle_gamedata_raw::tables::operator::{Operator, OperatorRarity};
use myrtle_types::Server;
use serde_json::json;

fn operators(entries: &[(&str, OperatorRarity)]) -> HashMap<String, Operator> {
    entries
        .iter()
        .map(|(id, rarity)| {
            let operator = Operator {
                rarity: rarity.clone(),
                ..Operator::default()
            };
            ((*id).to_owned(), operator)
        })
        .collect()
}

fn pair(char_id: &str, rarity: i16) -> CharRarityRow {
    CharRarityRow {
        char_id: char_id.to_owned(),
        rarity,
    }
}

fn fix(char_id: &str, from: i16, to: i16) -> RarityFix {
    RarityFix {
        char_id: char_id.to_owned(),
        from,
        to,
    }
}

#[test]
fn fixes_name_each_disagreeing_char_once_and_skip_unknown_ones() {
    let ops = operators(&[
        ("char_a", OperatorRarity::SixStar),
        ("char_b", OperatorRarity::FiveStar),
        ("char_c", OperatorRarity::ThreeStar),
    ]);
    let pairs = [
        pair("char_a", 3),
        pair("char_b", 5),
        pair("char_gone", 4),
        pair("char_c", 4),
        pair("char_a", 5),
        pair("char_a", 6),
    ];
    assert_eq!(
        rarity_fixes(&pairs, &ops),
        [fix("char_a", 3, 6), fix("char_c", 4, 3)]
    );
    assert!(rarity_fixes(&[pair("char_b", 5)], &ops).is_empty());
    assert!(rarity_fixes(&[], &ops).is_empty());
}

#[tokio::test]
async fn the_plan_reads_the_records_and_writes_nothing() {
    let (ctx, db) = common::context().await;
    let user = db.user(12, Server::EN).await;
    let pulls = json!([
        {"char_id": "char_a", "pool_id": "NORM_1", "rarity": 3, "pull_timestamp": 100,
         "pool_name": null, "gacha_type": "normal", "batch_index": 0},
        {"char_id": "char_b", "pool_id": "NORM_1", "rarity": 5, "pull_timestamp": 100,
         "pool_name": null, "gacha_type": "normal", "batch_index": 1},
    ]);
    let mut conn = ctx.db.conn().await.unwrap();
    let stored = gacha::insert_batch(&mut conn, user, &pulls).await.unwrap();
    assert_eq!(stored, gacha::BatchOutcome::Stored);
    drop(conn);
    let ops = operators(&[
        ("char_a", OperatorRarity::SixStar),
        ("char_b", OperatorRarity::FiveStar),
    ]);

    let plan = plan_against(&ctx, &ops).await.unwrap();
    assert_eq!(plan.distinct_pairs, 2);
    assert_eq!(plan.fixes, [fix("char_a", 3, 6)]);
    let again = plan_against(&ctx, &ops).await.unwrap();
    assert_eq!(again, plan, "a plan updates no row");
    db.close().await;
}
