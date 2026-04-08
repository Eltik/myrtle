/// Detailed grade breakdown for a user.
/// Shows per-operator, per-dimension scoring, base optimization, and roguelike progress.
///
/// Usage: cargo run --bin grade-detail -- --id <uuid or arknights_uid>
use std::collections::{HashMap, HashSet};
use std::path::Path;

use backend::core::gamedata::types::GameData;
use backend::core::gamedata::types::module::ModuleType;
use backend::core::gamedata::types::operator::{Operator, OperatorProfession, OperatorRarity};
use backend::core::gamedata::types::roguelike::RoguelikeGameData;
use backend::core::grade::base::assignment::compute_sustained_assignment;
use backend::core::grade::base::buff_registry::build_registry;
use backend::core::grade::base::score::grade_base;
use backend::core::grade::base::types::{OperatorBaseProfile, UserBuilding};
use backend::core::grade::calculate::calculate_user_grade;
use backend::core::grade::grade_roguelike::grade_roguelike;
use backend::database::models::roster::RosterEntry;
use backend::database::queries::{building, roguelike, roster, users};
use serde::Deserialize;
use uuid::Uuid;

// ── Helpers ────────────────────────────────────────────────────

fn score_to_grade(score: f64) -> &'static str {
    match score {
        s if s >= 0.90 => "S+",
        s if s >= 0.75 => "S",
        s if s >= 0.60 => "A",
        s if s >= 0.45 => "B",
        s if s >= 0.30 => "C",
        s if s >= 0.15 => "D",
        _ => "F",
    }
}

fn rarity_stars(r: &OperatorRarity) -> &'static str {
    match r {
        OperatorRarity::SixStar => "6★",
        OperatorRarity::FiveStar => "5★",
        OperatorRarity::FourStar => "4★",
        OperatorRarity::ThreeStar => "3★",
        OperatorRarity::TwoStar => "2★",
        OperatorRarity::OneStar => "1★",
    }
}

fn rarity_weight(r: &OperatorRarity) -> f64 {
    match r {
        OperatorRarity::SixStar => 1.0,
        OperatorRarity::FiveStar => 0.7,
        OperatorRarity::FourStar => 0.4,
        OperatorRarity::ThreeStar => 0.15,
        OperatorRarity::TwoStar => 0.1,
        OperatorRarity::OneStar => 0.05,
    }
}

fn op_name<'a>(id: &'a str, gd: &'a GameData) -> &'a str {
    gd.operators.get(id).map(|o| o.name.as_str()).unwrap_or(id)
}

fn log_curve(t: f64) -> f64 {
    (1.0 + t).ln() / 2.0_f64.ln()
}

async fn resolve_user_id(pool: &sqlx::PgPool, id_str: &str) -> Uuid {
    if let Ok(uuid) = id_str.parse::<Uuid>() {
        return uuid;
    }
    let profile = users::find_by_uid(pool, id_str)
        .await
        .expect("database error looking up uid")
        .unwrap_or_else(|| panic!("no user found with UID: {id_str}"));
    profile.id
}

// ── Operator detail ────────────────────────────────────────────

#[derive(Deserialize)]
struct MasteryEntry {
    mastery: i16,
}

#[derive(Deserialize)]
struct ModuleEntry {
    id: String,
    level: i16,
}

struct OpDetail {
    name: String,
    rarity: String,
    elite: i16,
    level: i16,
    score: f64,
    weight: f64,
    dims: Vec<(String, f64, f64)>, // (label, weight, score)
}

fn detail_operator(entry: &RosterEntry, op: &Operator) -> OpDetail {
    let max_elite = (op.phases.len() - 1) as f64;
    let num_skills = op.skills.len();
    let can_master = num_skills > 0 && op.phases.len() >= 3;
    let advanced_mods: Vec<_> = op
        .modules
        .iter()
        .filter(|m| m.module.module_type == ModuleType::Advanced)
        .collect();

    let mut dims: Vec<(String, f64, f64)> = vec![];

    // Elite
    if max_elite > 0.0 {
        let s = entry.elite as f64 / max_elite;
        dims.push((
            format!("Elite (E{}/E{})", entry.elite, max_elite as i32),
            25.0,
            s,
        ));
    }

    // Level
    let level_s = {
        let mut progress = 0.0;
        let mut total = 0.0;
        for (i, phase) in op.phases.iter().enumerate() {
            let ml = phase.max_level as f64;
            total += ml;
            if (i as i16) < entry.elite {
                progress += ml;
            } else if i as i16 == entry.elite {
                progress += entry.level as f64;
            }
        }
        if total > 0.0 {
            log_curve(progress / total)
        } else {
            1.0
        }
    };
    let lw = match op.rarity {
        OperatorRarity::SixStar => 15.0,
        OperatorRarity::FiveStar => 18.0,
        OperatorRarity::FourStar => 20.0,
        _ => 40.0,
    };
    dims.push((
        format!("Level (E{}L{})", entry.elite, entry.level),
        lw,
        level_s,
    ));

    // Skill level (only if can't master)
    let masteries: Vec<MasteryEntry> =
        serde_json::from_value(entry.masteries.clone()).unwrap_or_default();
    if !can_master && num_skills > 0 {
        let s = (entry.skill_level - 1) as f64 / 6.0;
        dims.push((format!("Skill Lv (SL{})", entry.skill_level), 20.0, s));
    }

    // Mastery
    if can_master {
        let m3 = masteries.iter().filter(|m| m.mastery >= 3).count();
        let mastery_s = if m3 == 0 {
            let total: f64 = masteries.iter().map(|m| m.mastery as f64).sum();
            let max = num_skills as f64 * 3.0;
            if max > 0.0 { (total / max) * 0.30 } else { 0.0 }
        } else {
            let base = match m3 {
                1 => 0.50,
                2 => 0.75,
                _ => 1.00,
            };
            let rem = num_skills - m3;
            if rem > 0 {
                let non_m3: f64 = masteries
                    .iter()
                    .filter(|m| m.mastery < 3)
                    .map(|m| m.mastery as f64)
                    .sum();
                (base + (non_m3 / (rem as f64 * 3.0)) * 0.10).min(1.0)
            } else {
                base
            }
        };
        let m_str: Vec<String> = masteries
            .iter()
            .map(|m| format!("M{}", m.mastery))
            .collect();
        dims.push((format!("Mastery ({})", m_str.join("/")), 30.0, mastery_s));
    }

    // Modules
    if !advanced_mods.is_empty() {
        let user_mods: Vec<ModuleEntry> =
            serde_json::from_value(entry.modules.clone()).unwrap_or_default();
        let adv_ids: HashSet<&str> = advanced_mods
            .iter()
            .map(|m| m.module.uni_equip_id.as_str())
            .collect();
        let user_adv: Vec<i16> = user_mods
            .iter()
            .filter(|m| adv_ids.contains(m.id.as_str()))
            .map(|m| m.level)
            .collect();
        let mod3 = user_adv.iter().filter(|&&l| l >= 3).count();
        let n = advanced_mods.len();
        let mod_s = if mod3 == 0 {
            let total: f64 = user_adv.iter().map(|&l| l as f64).sum();
            (total / (n as f64 * 3.0)) * 0.30
        } else {
            let base = (mod3 as f64 / n as f64).max(0.50);
            let rem = n - mod3;
            if rem > 0 {
                let non_max: f64 = user_adv.iter().filter(|&&l| l < 3).map(|&l| l as f64).sum();
                (base + (non_max / (rem as f64 * 3.0)) * 0.10).min(1.0)
            } else {
                base
            }
        };
        let mod_str: Vec<String> = user_adv.iter().map(|l| format!("L{l}")).collect();
        dims.push((format!("Modules ({})", mod_str.join("/")), 25.0, mod_s));
    }

    // Potential
    let pot_matters = !op.can_use_general_potential_item || op.is_sp_char;
    if pot_matters {
        let s = (entry.potential - 1) as f64 / 5.0;
        dims.push((format!("Potential (P{})", entry.potential), 10.0, s));
    }

    let tw: f64 = dims.iter().map(|(_, w, _)| w).sum();
    let score = dims.iter().map(|(_, w, s)| w * s).sum::<f64>() / tw;

    OpDetail {
        name: op.name.clone(),
        rarity: rarity_stars(&op.rarity).to_string(),
        elite: entry.elite,
        level: entry.level,
        score,
        weight: rarity_weight(&op.rarity),
        dims,
    }
}

// ── Roguelike detail ───────────────────────────────────────────

fn print_roguelike_detail(
    theme_progress: &[(String, serde_json::Value)],
    roguelike_data: &RoguelikeGameData,
) {
    println!("\n{}", "=".repeat(60));
    println!("=== ROGUELIKE (INTEGRATED STRATEGIES) ===");
    println!();

    if theme_progress.is_empty() {
        println!("  No roguelike data.");
        return;
    }

    let theme_names: HashMap<&str, &str> = [
        ("rogue_1", "Phantom & Crimson Solitaire"),
        ("rogue_2", "Mizuki & Caerula Arbor"),
        ("rogue_3", "Expeditioner's Joklumarkar"),
        ("rogue_4", "Sarkaz Reclamation"),
        ("rogue_5", "Chronicles of Babel"),
    ]
    .into();

    let theme_weights: HashMap<&str, f64> = [
        ("rogue_1", 0.12),
        ("rogue_2", 0.16),
        ("rogue_3", 0.20),
        ("rogue_4", 0.24),
        ("rogue_5", 0.28),
    ]
    .into();

    for (theme_id, progress) in theme_progress {
        let Some(td) = roguelike_data.themes.get(theme_id) else {
            continue;
        };
        let fallback = theme_id.as_str();
        let name = theme_names
            .get(theme_id.as_str())
            .copied()
            .unwrap_or(fallback);
        let w = theme_weights
            .get(theme_id.as_str())
            .copied()
            .unwrap_or(0.20);

        // Parse user progress fields
        let record = progress.get("record");
        let bp = progress.get("bp");
        let collect = progress.get("collect");
        let challenge = progress.get("challenge");

        // Endings
        let unique_endings = {
            let mut ids = HashSet::new();
            if let Some(ec) = record
                .and_then(|r| r.get("endingCnt"))
                .and_then(|e| e.as_object())
            {
                for mode_val in ec.values() {
                    if let Some(ends) = mode_val.as_object() {
                        for k in ends.keys() {
                            ids.insert(k.clone());
                        }
                    }
                }
            }
            ids.len() as i32
        };

        // Difficulty
        let highest_cleared = collect
            .and_then(|c| c.get("modeGrade"))
            .and_then(|mg| mg.get("NORMAL"))
            .and_then(|n| n.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(grade_str, entry)| {
                        let state = entry.get("state")?.as_i64()?;
                        if state >= 2 {
                            grade_str.parse::<i32>().ok()
                        } else {
                            None
                        }
                    })
                    .max()
                    .unwrap_or(-1)
            })
            .unwrap_or(-1);

        // Collectibles
        let count_unlocked = |key: &str| -> usize {
            collect
                .and_then(|c| c.get(key))
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.values()
                        .filter(|v| {
                            v.get("state")
                                .and_then(|s| s.as_i64())
                                .is_some_and(|s| s >= 1)
                        })
                        .count()
                })
                .unwrap_or(0)
        };
        let relics = count_unlocked("relic");
        let capsules = count_unlocked("capsule");
        let bands = count_unlocked("band");
        // Cap per-category to max (user data includes non-catalog items)
        let relics_capped = relics.min(td.max_relics as usize);
        let capsules_capped = capsules.min(td.max_capsules as usize);
        let bands_capped = bands.min(td.max_bands as usize);
        let max_coll = td.max_relics + td.max_capsules + td.max_bands;
        let total_coll = (relics_capped + capsules_capped + bands_capped) as i32;

        // BP
        let bp_level = bp
            .and_then(|b| b.get("reward"))
            .and_then(|r| r.as_object())
            .map(|o| o.len() as i32)
            .unwrap_or(0);

        // Challenges
        let challenges_done = challenge
            .and_then(|c| c.get("grade"))
            .and_then(|g| g.as_object())
            .map(|o| o.len() as i32)
            .unwrap_or(0);

        // Compute dimension scores
        let ending_s = if td.max_endings > 0 {
            (unique_endings as f64 / td.max_endings as f64).min(1.0)
        } else {
            0.0
        };
        let diff_s = if td.max_difficulty_grade > 0 && highest_cleared > 0 {
            let ratio = highest_cleared as f64 / td.max_difficulty_grade as f64;
            if ratio >= 1.0 {
                1.0
            } else if ratio >= 0.75 {
                0.75
            } else if ratio >= 0.50 {
                0.50
            } else {
                0.25
            }
        } else {
            0.0
        };
        let coll_s = if max_coll > 0 {
            (total_coll as f64 / max_coll as f64).min(1.0)
        } else {
            0.0
        };
        let bp_s = if td.max_bp_levels > 0 {
            log_curve((bp_level as f64 / td.max_bp_levels as f64).min(1.0))
        } else {
            0.0
        };
        let ch_s = if td.max_challenges > 0 {
            (challenges_done as f64 / td.max_challenges as f64).min(1.0)
        } else {
            0.0
        };

        // Weighted theme score
        let mut dims: Vec<(&str, f64, f64)> = vec![];
        dims.push(("Endings", 30.0, ending_s));
        dims.push(("Difficulty", 25.0, diff_s));
        let coll_w = if td.max_challenges == 0 { 30.0 } else { 20.0 };
        dims.push(("Collectibles", coll_w, coll_s));
        dims.push(("BP Progress", 15.0, bp_s));
        if td.max_challenges > 0 {
            dims.push(("Challenges", 10.0, ch_s));
        }
        let tw: f64 = dims.iter().map(|(_, w, _)| w).sum();
        let theme_score = dims.iter().map(|(_, w, s)| w * s).sum::<f64>() / tw;

        println!(
            "  --- {} ({}) [weight {:.0}%] ---",
            name,
            theme_id,
            w * 100.0
        );
        println!(
            "    Endings:      {}/{:<4}  score {:.2}",
            unique_endings, td.max_endings, ending_s
        );
        println!(
            "    Difficulty:   {}/{:<4}  score {:.2} (milestone)",
            highest_cleared.max(0),
            td.max_difficulty_grade,
            diff_s
        );

        // Show capped collectibles, with raw counts if they differ
        let relic_display = if relics > relics_capped {
            format!("{} (raw {})", relics_capped, relics)
        } else {
            relics_capped.to_string()
        };
        let capsule_display = if capsules > capsules_capped {
            format!("{} (raw {})", capsules_capped, capsules)
        } else {
            capsules_capped.to_string()
        };
        let band_display = if bands > bands_capped {
            format!("{} (raw {})", bands_capped, bands)
        } else {
            bands_capped.to_string()
        };
        println!(
            "    Collectibles: {}/{:<4}  score {:.2}  (relics {}, capsules {}, bands {})",
            total_coll, max_coll, coll_s, relic_display, capsule_display, band_display
        );

        println!(
            "    BP Level:     {}/{:<4}  score {:.2} (log compressed)",
            bp_level, td.max_bp_levels, bp_s
        );
        if td.max_challenges > 0 {
            println!(
                "    Challenges:   {}/{:<4}  score {:.2}",
                challenges_done, td.max_challenges, ch_s
            );
        }
        println!("    Theme Score:  {:.4}", theme_score);
        println!();
    }
}

// ── Main ───────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let mut args = std::env::args().skip(1);
    let id_str = match args.next().as_deref() {
        Some("--id") => args.next().expect("--id requires a value"),
        Some(other) => panic!("unknown argument: {other}"),
        None => panic!("usage: grade-detail --id <uuid or arknights_uid>"),
    };

    let data_dir =
        std::env::var("GAME_DATA_DIR").unwrap_or_else(|_| "../assets/output/gamedata/excel".into());
    let assets_dir = std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output".into());

    println!("Loading game data...");
    let game_data =
        backend::core::gamedata::init_game_data(Path::new(&data_dir), Path::new(&assets_dir))
            .expect("failed to load game data");

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = backend::database::init(&db_url)
        .await
        .expect("failed to init database");
    let user_id = resolve_user_id(&db, &id_str).await;

    // Fetch all data
    let user_roster = roster::get_roster(&db, user_id).await.expect("roster");
    let building_json = building::get_building(&db, user_id)
        .await
        .expect("building");
    let rl_progress = roguelike::get_roguelike_progress(&db, user_id)
        .await
        .expect("roguelike");

    // ── Overall grade ──────────────────────────────────────────
    let grade = calculate_user_grade(&db, user_id, &game_data)
        .await
        .expect("grade");

    println!("\n============================================================");
    println!(
        "  OVERALL GRADE: {}  (score {:.4})",
        grade.overall, grade.total_score
    );
    println!("    Operators: {:.4}  (weight 1.0)", grade.operator_grade);
    println!("    Base:      {:.4}  (weight 0.5)", grade.base_grade);
    println!("    Roguelike: {:.4}  (weight 0.3)", grade.roguelike_grade);
    println!("============================================================");

    // ── Operator grade detail ──────────────────────────────────
    println!("\n=== OPERATOR GRADE DETAIL ===");
    println!();

    let roster_map: HashMap<&str, &RosterEntry> = user_roster
        .iter()
        .map(|r| (r.operator_id.as_str(), r))
        .collect();

    let mut details: Vec<OpDetail> = Vec::new();
    for (op_id, static_op) in &game_data.operators {
        if matches!(
            static_op.profession,
            OperatorProfession::Token | OperatorProfession::Trap
        ) {
            continue;
        }
        if static_op.is_not_obtainable {
            continue;
        }
        let Some(entry) = roster_map.get(op_id.as_str()) else {
            continue;
        };
        if entry.elite == 0 && entry.level <= 1 {
            continue;
        }

        details.push(detail_operator(entry, static_op));
    }

    // Sort by weighted contribution (weight * score) descending
    details.sort_by(|a, b| {
        (b.weight * b.score)
            .partial_cmp(&(a.weight * a.score))
            .unwrap()
    });

    // Show top contributors
    let top_n = 20;
    println!("  Top {top_n} contributors (by weighted score):");
    println!(
        "  {:<30} {:>5} {:>7} {:>7} {:>8}",
        "Operator", "Rarity", "Score", "Weight", "Contrib"
    );
    println!("  {}", "-".repeat(62));
    for d in details.iter().take(top_n) {
        println!(
            "  {:<30} {:>5} {:>7.4} {:>7.2} {:>8.4}",
            format!("{} (E{}L{})", d.name, d.elite, d.level),
            d.rarity,
            d.score,
            d.weight,
            d.weight * d.score,
        );
        for (label, w, s) in &d.dims {
            println!("    {:<34} w={:<5.0} s={:.4}", label, w, s);
        }
    }

    // Show bottom operators
    println!("\n  Bottom {top_n} (lowest scores with investment):");
    println!(
        "  {:<30} {:>5} {:>7} {:>7} {:>8}",
        "Operator", "Rarity", "Score", "Weight", "Contrib"
    );
    println!("  {}", "-".repeat(62));
    for d in details.iter().rev().take(top_n) {
        println!(
            "  {:<30} {:>5} {:>7.4} {:>7.2} {:>8.4}",
            format!("{} (E{}L{})", d.name, d.elite, d.level),
            d.rarity,
            d.score,
            d.weight,
            d.weight * d.score,
        );
        for (label, w, s) in &d.dims {
            println!("    {:<34} w={:<5.0} s={:.4}", label, w, s);
        }
    }

    // Stats summary
    let total_weight: f64 = details.iter().map(|d| d.weight).sum();
    let total_score: f64 = details.iter().map(|d| d.weight * d.score).sum();
    let avg = if total_weight > 0.0 {
        total_score / total_weight
    } else {
        0.0
    };
    println!("\n  Operators graded: {}", details.len());
    println!("  Weighted average: {:.4} ({})", avg, score_to_grade(avg));

    // Rarity breakdown
    let mut by_rarity: HashMap<String, (usize, f64)> = HashMap::new();
    for d in &details {
        let e = by_rarity.entry(d.rarity.clone()).or_default();
        e.0 += 1;
        e.1 += d.score;
    }
    println!("\n  By rarity:");
    let mut rarity_list: Vec<_> = by_rarity.iter().collect();
    rarity_list.sort_by(|a, b| b.0.cmp(a.0)); // sort by rarity name desc (6★ first)
    for (rarity, (count, total)) in &rarity_list {
        let avg = total / *count as f64;
        println!("    {rarity}: {count} operators, avg score {avg:.4}");
    }

    // ── Base grade detail ──────────────────────────────────────
    println!("\n=== BASE GRADE DETAIL ===");

    if let Some(bj) = &building_json {
        let profiles: Vec<OperatorBaseProfile> = user_roster
            .iter()
            .filter_map(|e| {
                let bc = game_data.building.chars.get(&e.operator_id)?;
                Some(OperatorBaseProfile::build(e, bc))
            })
            .collect();
        let (registry, morale_drains) = build_registry(&game_data.building.buffs);
        let user_building = UserBuilding::from_json(bj);

        // Layout
        println!("\n  Layout:");
        let mut layout: HashMap<&str, Vec<i32>> = HashMap::new();
        for room in &user_building.rooms {
            layout.entry(&room.room_type).or_default().push(room.level);
        }
        let mut sorted: Vec<_> = layout.iter().collect();
        sorted.sort_by_key(|(n, _)| *n);
        for (rt, levels) in &sorted {
            let ls: Vec<String> = levels.iter().map(|l| format!("L{l}")).collect();
            println!("    {rt}: {} ({})", levels.len(), ls.join(", "));
        }

        // Optimal + sustained assignment
        println!("\n  Sustained Assignment (Shift A):");
        let sustained = compute_sustained_assignment(
            &profiles,
            &user_building,
            &game_data.building,
            &registry,
            &morale_drains,
        );
        for room in &sustained.shift_a.rooms {
            let ops: Vec<String> = room
                .operators
                .iter()
                .map(|id| format!("{} ({id})", op_name(id, &game_data)))
                .collect();
            let fl = room.formula_type.as_deref().unwrap_or("");
            println!(
                "    {} (L{}) {fl} — +{:.1}%",
                room.room_type, room.level, room.total_efficiency
            );
            for o in &ops {
                println!("      - {o}");
            }
        }
        println!(
            "    Shift A Total: +{:.1}%",
            sustained.shift_a.total_production_efficiency
        );

        println!("\n  Rotation (Shift B):");
        for room in &sustained.shift_b.rooms {
            let ops: Vec<String> = room
                .operators
                .iter()
                .map(|id| format!("{} ({id})", op_name(id, &game_data)))
                .collect();
            let fl = room.formula_type.as_deref().unwrap_or("");
            println!(
                "    {} (L{}) {fl} — +{:.1}%",
                room.room_type, room.level, room.total_efficiency
            );
            for o in &ops {
                println!("      - {o}");
            }
        }
        println!(
            "    Shift B Total: +{:.1}%",
            sustained.shift_b.total_production_efficiency
        );
        println!(
            "    Sustained Average: +{:.1}%",
            sustained.sustained_efficiency
        );

        let base_score = grade_base(&user_roster, Some(bj), &game_data);
        println!(
            "\n  Base Score: {base_score:.4} ({})",
            score_to_grade(base_score)
        );
    } else {
        println!("  No building data synced.");
    }

    // ── Roguelike grade detail ──────────────────────────────────
    println!("\n=== ROGUELIKE GRADE DETAIL ===");
    print_roguelike_detail(&rl_progress, &game_data.roguelike);

    let rl_score = grade_roguelike(&rl_progress, &game_data.roguelike);
    println!(
        "  Roguelike Score: {rl_score:.4} ({})",
        score_to_grade(rl_score)
    );
}
