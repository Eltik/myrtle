//! Operator-file tables: attributes, filters over the roster and real names.

use std::collections::{BTreeSet, HashSet};

use serde_json::Value;

use super::text::{contained_value, norm};
use super::{CLASSES, PLACE_KEYS, Field, Filter, Route, ToolError, ToolResult, Tools, s};

impl Tools {
    pub(super) fn attrs(&self) -> Result<&Vec<Value>, ToolError> {
        self.attributes.as_ref().ok_or(ToolError::Missing("operator_attributes.jsonl"))
    }

    /// Values of one attribute column, without the game's placeholders (the routes' dictionaries).
    #[must_use]
    pub fn attr_values(&self, k: &str) -> BTreeSet<String> {
        self.attributes.iter().flatten().map(|r| s(r, k)).filter(|v| v.len() >= 3 && !matches!(v.as_str(), "Unknown" | "Undisclosed"))
            .collect()
    }

    /// Affiliations and birthplaces, the values a place filter takes.
    #[must_use]
    pub fn places(&self) -> BTreeSet<String> {
        PLACE_KEYS.iter().flat_map(|k| self.attr_values(k)).collect()
    }

    pub(super) fn names_table(&self) -> Result<&Vec<Value>, ToolError> {
        self.real_names.as_ref().ok_or(ToolError::Missing("real_names.jsonl"))
    }

    /// One operator's real name with the line that states it (`scripts/real_names.py`).
    ///
    /// # Errors
    /// Missing table, an unknown operator, or no real name in the table (retrieval may still find one).
    pub fn real_name(&self, operator: &str) -> ToolResult {
        let rows = self.names_table()?;
        let names: Vec<&str> = rows.iter().filter_map(|r| r["name"].as_str()).filter(|n| !n.is_empty()).collect();
        let who = self.resolve_name("operator", operator, &names)?;
        let r = rows.iter().rev().find(|r| r["name"].as_str() == Some(who)).ok_or_else(|| ToolError::NotFound { what: "operator", name: operator.to_owned() })?;
        r["realName"].is_string().then(|| format!("{}'s real name is {}. The line that states it: \"{}\" ({}).",
                                                  s(r, "name"), s(r, "realName"), s(r, "quote"), s(r, "storyId")))
            .ok_or_else(|| ToolError::NoData(format!("no real name for {who} in the table")))
    }

    /// Every real name in the table, optionally for operators affiliated with or born in `place`.
    ///
    /// # Errors
    /// Missing table or an unknown place.
    pub fn real_names(&self, place: Option<&str>) -> ToolResult {
        let rows = self.names_table()?;
        let named: Vec<&Value> = rows.iter().filter(|r| r["realName"].is_string()).collect();
        let line = |r: &Value| format!("- {}: {} (\"{}\", {})", s(r, "name"), s(r, "realName"), s(r, "quote"), s(r, "storyId"));
        // "Real names of every operator from Kazimierz": the place filter of the attribute table narrows the list.
        let (place, ids) = match place {
            Some(p) => {
                let p = self.resolve_place(p)?;
                let ids: HashSet<String> = self.attrs()?.iter().filter(|r| PLACE_KEYS.iter().any(|k| r[*k].as_str() == Some(p.as_str())))
                    .filter_map(|r| r["charId"].as_str().map(str::to_owned)).collect();
                (Some(p), Some(ids))
            }
            None => (None, None),
        };
        let named: Vec<&&Value> = named.iter().filter(|r| ids.as_ref().is_none_or(|ids| ids.contains(&s(r, "charId")))).collect();
        let scope = place.map_or(String::new(), |p| format!(" affiliated with or born in {p}"));
        let mut out = format!("Real names Trevor found for {} playable operators{scope} (of {} checked), each with the line that \
states it. Checked: each operator's file and up to 4 story passages that name them near a naming word, so a name revealed \
elsewhere can be missing; for the rest, those passages give no real name.\n", named.len(), rows.len());
        for r in &named {
            out.push_str(&line(r)); out.push('\n');
        }
        Ok(out)
    }

    // ---------------------------------------------------------------- game-data attributes
    // Roster questions answered from the game data (`scripts/attributes.py`): "Which operators are Sarkaz?",
    // "How many 6-star Casters are there?", "List all operators from Kazimierz", "What race is Texas?".

    pub(super) fn resolve_place(&self, p: &str) -> Result<String, ToolError> {
        let places: Vec<String> = self.places().into_iter().collect();
        let refs: Vec<&str> = places.iter().map(String::as_str).collect();
        self.resolve_name("place", p, &refs).or_else(|e| contained_value(p, &refs).ok_or(e)).map(str::to_owned)
    }

    /// A value of `k` (race, branch), accepting a plural ("Sarkazs") or a value inside a longer text ("Fortress
    /// defenders": the model router passed the branch with its class on 2026-09-29).
    pub(super) fn resolve_value(&self, what: &'static str, k: &str, v: &str) -> Result<String, ToolError> {
        let vals: Vec<String> = self.attr_values(k).into_iter().collect();
        let refs: Vec<&str> = vals.iter().map(String::as_str).collect();
        self.resolve_name(what, v, &refs)
            .or_else(|e| v.strip_suffix('s').map_or(Err(e.clone()), |x| self.resolve_name(what, x, &refs).map_err(|_| e)))
            .or_else(|e| contained_value(v, &refs).ok_or(e))
            .map(str::to_owned)
    }

    /// Operators matching every filter, listed with a count, or the 5 tallest or shortest of them.
    ///
    /// # Errors
    /// Missing table, no filter and no sort, or a value no operator has.
    pub fn operator_filter(&self, f: &Filter) -> ToolResult {
        let rows = self.attrs()?;
        type Pred = Box<dyn Fn(&Value) -> bool>;
        let mut filters: Vec<(String, Pred)> = Vec::new();
        // A value in the wrong field moves to the field it resolves in: the model router wrote {"place": "Soloblade"}
        // for "List all Soloblade operators" (2026-09-29). Only a place that is no place moves.
        let mut f = f.clone();
        if let Some(p) = f.place.clone().filter(|p| self.resolve_place(p).is_err()) {
            if f.branch.is_none() && self.resolve_value("branch", "branch", &p).is_ok() {
                f.branch = f.place.take();
            } else if f.race.is_none() && self.resolve_value("race", "race", &p).is_ok() {
                f.race = f.place.take();
            }
        }
        let f = &f;
        if let Some(p) = &f.place {
            let p = self.resolve_place(p)?;
            let pl = p.clone();
            filters.push((format!("affiliated with or born in {p}"), Box::new(move |r| PLACE_KEYS
                .iter().any(|k| r[*k].as_str() == Some(pl.as_str())))));
        }
        if let Some(race) = &f.race {
            let race = self.resolve_value("race", "race", race)?;
            let rl = race.clone();
            filters.push((format!("race {race}"), Box::new(move |r| r["race"].as_str().is_some_and(|x| x.split('/').any(|p| p.trim() == rl)))));
        }
        for c in &f.classes {
            let c = resolve_class(c)?;
            filters.push((format!("class {c}"), Box::new(move |r| r["class"].as_str() == Some(c))));
        }
        if let Some(b) = &f.branch {
            let b = self.resolve_value("branch", "branch", b)?;
            let bl = b.clone();
            filters.push((format!("branch {b}"), Box::new(move |r| r["branch"].as_str() == Some(bl.as_str()))));
        }
        if let Some(n) = f.rarity {
            if !(1..=6).contains(&n) {
                return Err(ToolError::BadArg { arg: "rarity", value: n.to_string() });
            }
            filters.push((format!("{n}-star"), Box::new(move |r| r["rarity"].as_i64() == Some(n))));
        }
        match f.female {
            Some(true) => filters.push(("female".into(), Box::new(|r| r["gender"].as_str() == Some("Female")))),
            Some(false) => filters.push(("male".into(), Box::new(|r| r["gender"].as_str() == Some("Male")))),
            None => {}
        }
        match f.infected {
            Some(false) => filters.push(("not infected".into(), Box::new(|r| r["infected"].as_bool() == Some(false)))),
            Some(true) => filters.push(("infected".into(), Box::new(|r| r["infected"].as_bool() == Some(true)))),
            None => {}
        }
        if filters.is_empty() && f.tallest.is_none() {
            return Err(ToolError::MissingArg("a filter or sort"));
        }
        let hit: Vec<&Value> = rows.iter().filter(|r| filters.iter().all(|(_, f)| f(r))).collect();
        let desc = filters.iter().map(|(d, _)| d.as_str()).collect::<Vec<_>>().join(", ");
        // Superlatives over height, the one ordered attribute the game data gives every operator (401 of 407).
        if let Some(tallest) = f.tallest {
            let w = if tallest { "tallest" } else { "shortest" };
            let mut h: Vec<&Value> = hit.into_iter().filter(|r| r["heightCm"].is_i64()).collect();
            h.sort_by_key(|r| r["heightCm"].as_i64().unwrap_or(0));
            if tallest {
                h.reverse();
            }
            let scope = if desc.is_empty() { "playable operators".to_owned() } else { format!("operators ({desc})") };
            let mut out = format!("The {w} {scope} by the height in their operator files ({} with a height). The files \
give the height as written, some approximate or in armor.\n", h.len());
            for r in h.iter().take(5) {
                out.push_str(&format!("- {}: {} cm\n", s(r, "name"), r["heightCm"].as_i64().unwrap_or(0)));
            }
            return Ok(out);
        }
        let mut out = format!("{} of the {} playable operators are {desc}, from the game data (character_table and each \
operator's file). The game leaves some fields Unknown or Undisclosed; those operators are not counted.\n", hit.len(), rows.len());
        for r in &hit {
            let why = if desc.contains("affiliated") {
                let aff = ["nation", "group", "team"].iter().map(|k| s(r, k)).filter(|v| !v.is_empty()).collect::<Vec<_>>().join(", ");
                format!(" (affiliation: {}; born in {})", if aff.is_empty() { "none".into() } else { aff },
                        Some(s(r, "birthplace")).filter(|b| !b.is_empty()).unwrap_or_else(|| "not given".into()))
            } else { String::new() };
            out.push_str(&format!("- {}{why}\n", s(r, "name")));
        }
        Ok(out)
    }

    /// One operator's attribute: "What race is Texas?", "When is Exusiai's birthday?".
    ///
    /// # Errors
    /// Missing table, an unknown operator, or a field the game leaves empty for them.
    pub fn operator_attribute(&self, operator: &str, field: Field) -> ToolResult {
        let rows = self.attrs()?;
        let names = self.operator_names();
        let who = self.resolve_name("operator", operator, &names)?;
        let r = rows.iter().rev().find(|r| r["name"].as_str() == Some(who)).ok_or_else(|| ToolError::NotFound { what: "operator", name: operator.to_owned() })?;
        let v = match field.key() {
            "heightCm" => r["heightCm"].as_i64().map(|h| format!("{h} cm")),
            "rarity" => r["rarity"].as_i64().map(|h| format!("{h} stars")),
            "nation" => Some(["nation", "group", "team"].iter().map(|k| s(r, k)).filter(|v| !v.is_empty()).collect::<Vec<_>>().join(", "))
                .filter(|v| !v.is_empty()),
            k => r[k].as_str().map(str::to_owned),
        }.ok_or_else(|| ToolError::NoData(format!("{who} has no {}", field.key())))?;
        let label = match field.key() { "heightCm" => "height", "infectionText" => "infection status", "nation" => "affiliation", k => k };
        Ok(format!("{}'s {label}, from their operator file in the game data: {v}.", s(r, "name")))
    }
}

pub(super) fn resolve_class(c: &str) -> Result<&'static str, ToolError> {
    let n = norm(c);
    CLASSES.iter().find(|k| { let k = k.to_lowercase(); n == k || n == format!("{k}s") }).copied()
        .ok_or_else(|| ToolError::BadArg { arg: "class", value: c.to_owned() })
}

/// The filter of an `operator_filter` route. `class` may hold several classes separated by commas.
///
/// # Errors
/// A value that is not a number, a boolean, a gender or a sort.
pub fn filter_from_args(r: &Route) -> Result<Filter, ToolError> {
    let bool_arg = |k: &'static str| -> Result<Option<bool>, ToolError> {
        r.opt(k).map(|v| match v.to_lowercase().as_str() {
            "true" | "yes" | "infected" => Ok(true),
            "false" | "no" | "not infected" | "uninfected" => Ok(false),
            _ => Err(ToolError::BadArg { arg: k, value: v.to_owned() }),
        }).transpose()
    };
    Ok(Filter {
        place: r.opt("place").map(str::to_owned),
        race: r.opt("race").map(str::to_owned),
        classes: r.opt("class").map(|c| c.split(',').map(|x| x.trim().to_owned()).filter(|x| !x.is_empty()).collect()).unwrap_or_default(),
        branch: r.opt("branch").map(str::to_owned),
        rarity: r.opt("rarity").map(|v| v.trim_end_matches(['*', ' ']).trim_end_matches("-star").trim_end_matches(" star").parse::<i64>()
            .map_err(|_| ToolError::BadArg { arg: "rarity", value: v.to_owned() })).transpose()?,
        female: r.opt("gender").map(|v| match v.to_lowercase().as_str() {
            "female" | "women" | "woman" => Ok(true),
            "male" | "men" | "man" => Ok(false),
            _ => Err(ToolError::BadArg { arg: "gender", value: v.to_owned() }),
        }).transpose()?,
        infected: bool_arg("infected")?,
        tallest: r.opt("sort").map(|v| match v.to_lowercase().as_str() {
            "tallest" => Ok(true),
            "shortest" => Ok(false),
            _ => Err(ToolError::BadArg { arg: "sort", value: v.to_owned() }),
        }).transpose()?,
    })
}
