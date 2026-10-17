//! Questions about events past a reader's spoiler horizon (`--serve` and `--worker` jobs with a horizon).
//!
//! - The spoiler notice (`--no-spoiler-notice`): a question naming an event (`Tools::event_in_question`) whose every
//!   story lies after the horizon is answered with `prompts.rs` SPOILER_NOTICE, with no retrieval and no model call,
//!   so nothing of the event is served. Before it, "Give me a summary of The Masses Travels" at Shatterpoint retrieved
//!   passages within the horizon, none of them from the event, and the answer said they held no summary.
//! - The recap under a horizon (`--no-horizon-recap`): a recap route (the event summary table) is served when every
//!   story of its event lies at or before the horizon; every other table answer stays left out under a horizon.

use trevor::search::horizon::Horizon;
use trevor::tools::{Route, Tools};

use crate::detect::has;
use crate::prompts::{SPOILER_ACT, SPOILER_ACT_SUMMARY, SPOILER_NOTICE, SPOILER_WHERE};

/// A summary request: "summary", "summarise", "recap", "what happens in" and the like.
pub(crate) fn asks_summary(q: &str) -> bool {
    has(q, &["summary", "summarize", "summarise", "recap", "what happens in", "what happened in", "plot of",
             "rundown of", "tl dr", "tldr"])
}

/// The spoiler notice for `q` at horizon `h`, when the question names an event whose first story lies after it.
pub(crate) fn notice(q: &str, tools: &Tools, h: &Horizon) -> Option<String> {
    let g = tools.event_in_question(q)?;
    let gid = g["groupId"].as_str()?;
    let (first, _) = h.order().group_span(gid)?;
    if first <= h.pos {
        return None;
    }
    let name = g["name"].as_str().unwrap_or(gid);
    let at = h.order().era_at(h.pos).map(|e| SPOILER_WHERE.replace("{era}", &e.name)).unwrap_or_default();
    let act = if asks_summary(q) { SPOILER_ACT_SUMMARY } else { SPOILER_ACT };
    Some(SPOILER_NOTICE.replace("{event}", name).replace("{where}", &at).replace("{act}", act))
}

/// Whether `route` is a recap of an event whose every story lies at or before `h`.
pub(crate) fn recap_within(route: &Route, tools: &Tools, h: &Horizon) -> bool {
    if route.tool != "recap" {
        return false;
    }
    let Some(g) = route.args.get("event").and_then(|e| tools.resolve_event(e).ok()) else { return false };
    let Some(gid) = g["groupId"].as_str() else { return false };
    // The keyword router's recap reads the ending's stories by a substring test on story ids ("main_1" also takes
    // main_10 to main_16): an ending recap of such a group is not served under a horizon.
    let ending = route.args.get("ending").is_some_and(|v| v.eq_ignore_ascii_case("true"));
    if ending && tools.groups.values().filter_map(|x| x["groupId"].as_str()).any(|x| x != gid && x.contains(gid)) {
        return false;
    }
    h.order().group_span(gid).is_some_and(|(_, last)| last <= h.pos)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;
    use trevor::search::horizon::ReleaseOrder;

    use super::*;

    fn setup() -> (Tools, Arc<ReleaseOrder>) {
        let guide = json!({"items": [
            {"groupId": "main_0", "position": 1, "released": "by 2020-01-22", "kind": "main", "name": "Ep 0"},
            {"groupId": "act16side", "position": 2, "released": "by 2022-09-16", "kind": "event", "name": "Guide Ahead"},
            {"groupId": "act42side", "position": 3, "released": "by 2025-10-14", "kind": "event",
             "name": "The Masses' Travels"}
        ]});
        let stories = vec![
            json!({"storyId": "main_0_a", "groupId": "main_0", "storySort": 1}),
            json!({"storyId": "act16side_a", "groupId": "act16side", "storySort": 1}),
            json!({"storyId": "act16side_b", "groupId": "act16side", "storySort": 2}),
            json!({"storyId": "act42side_a", "groupId": "act42side", "storySort": 1}),
        ];
        let mut t = Tools::default();
        for g in guide["items"].as_array().unwrap() {
            t.groups.insert(g["groupId"].as_str().unwrap().to_owned(), g.clone());
        }
        (t, Arc::new(ReleaseOrder::from_parts(&guide, &stories)))
    }

    #[test]
    fn an_event_wholly_past_the_horizon_gets_the_notice() {
        let (t, o) = setup();
        let h = o.horizon_at(3).unwrap();
        let n = notice("Give me a summary of The Masses Travels", &t, &h).unwrap();
        assert!(n.starts_with("The Masses' Travels comes after where you have read"), "{n}");
        assert!(n.contains("(you are up to Ep 0), so Trevor won't summarise it yet. \
                            Turn off spoiler protection to ask about it."), "{n}");
        let n = notice("Who does Mon3tr suggest assassinating in masses travels?", &t, &h).unwrap();
        assert!(n.contains("won't answer questions about it yet"), "{n}");
        // Within the horizon, or partly read: no notice.
        assert!(notice("Summarize Guide Ahead", &t, &h).is_none());
        assert!(notice("Summarize Guide Ahead", &t, &o.horizon_at(2).unwrap()).is_none());
        assert!(notice("Who is Amiya?", &t, &h).is_none());
        assert!(notice("Give me a summary of The Masses Travels", &t, &o.horizon_at(4).unwrap()).is_none());
    }

    #[test]
    fn a_recap_is_served_only_for_an_event_wholly_within_the_horizon() {
        let (t, o) = setup();
        let r = |e: &str| Route::new("recap", &[("event", e), ("ending", "false")]);
        assert!(recap_within(&r("act16side"), &t, &o.horizon_at(3).unwrap()));
        assert!(!recap_within(&r("act16side"), &t, &o.horizon_at(2).unwrap()));
        assert!(!recap_within(&r("act42side"), &t, &o.horizon_at(3).unwrap()));
        assert!(!recap_within(&Route::new("timeline_all", &[]), &t, &o.horizon_at(4).unwrap()));
    }
}
