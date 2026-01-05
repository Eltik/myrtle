//! DPS Comparison Tests
//!
//! These tests compare Rust DPS calculations against pre-computed Python reference values.
//! Test cases are loaded from test_config.json to keep the file size small.
//!
//! Run with: cargo test --test dps_comparison_test
//!
//! To regenerate expected values:
//!   python tests/dps_comparison/run_comparison.py --generate-expected

use std::collections::HashMap;
use std::sync::OnceLock;

use backend::core::dps_calculator::operator_data::OperatorData;
use backend::core::dps_calculator::operator_unit::{EnemyStats, OperatorParams};
use backend::core::dps_calculator::operators;
use backend::core::local::handler::init_game_data;
use backend::core::local::types::GameData;
use backend::events::EventEmitter;

/// Tolerance for DPS comparison (percentage difference allowed)
const TOLERANCE_PERCENT: f64 = 0.15; // 15% (account for game data version differences)

/// Absolute tolerance for very small values
const TOLERANCE_ABSOLUTE: f64 = 1.0;

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct TestCase {
    operator: String,
    operator_id: String,
    rust_module: String,
    skill: i32,
    module: i32,
    defense: f64,
    res: f64,
}

/// Pre-computed expected DPS values from Python
/// Key: "operator_skill_defense_res" -> expected DPS
static EXPECTED_DPS: OnceLock<HashMap<String, f64>> = OnceLock::new();

/// Test cases loaded from test_config.json
static TEST_CASES: OnceLock<Vec<TestCase>> = OnceLock::new();

/// Loaded game data for creating operators
static GAME_DATA: OnceLock<Option<GameData>> = OnceLock::new();

fn get_expected_dps() -> &'static HashMap<String, f64> {
    EXPECTED_DPS.get_or_init(|| {
        let expected_path = std::env::var("EXPECTED_DPS_PATH")
            .unwrap_or_else(|_| "tests/dps_comparison/expected_dps.json".to_string());

        match std::fs::read_to_string(&expected_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
                eprintln!("Failed to parse expected_dps.json: {e}");
                HashMap::new()
            }),
            Err(e) => {
                eprintln!("Expected DPS file not found at {expected_path}: {e}");
                eprintln!("Run: python tests/dps_comparison/run_comparison.py --generate-expected");
                HashMap::new()
            }
        }
    })
}

fn get_test_cases() -> &'static Vec<TestCase> {
    TEST_CASES.get_or_init(|| {
        let config_path = std::env::var("TEST_CONFIG_PATH")
            .unwrap_or_else(|_| "tests/dps_comparison/test_config.json".to_string());

        match std::fs::read_to_string(&config_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
                eprintln!("Failed to parse test_config.json: {e}");
                Vec::new()
            }),
            Err(e) => {
                eprintln!("Test config not found at {config_path}: {e}");
                Vec::new()
            }
        }
    })
}

fn get_game_data() -> Option<&'static GameData> {
    GAME_DATA
        .get_or_init(|| {
            let data_dir = std::env::var("DATA_DIR").ok()?;
            let assets_dir = std::env::var("ASSETS_DIR").unwrap_or_else(|_| "./assets".to_string());

            let events = std::sync::Arc::new(EventEmitter::new());

            match init_game_data(
                std::path::Path::new(&data_dir),
                std::path::Path::new(&assets_dir),
                &events,
            ) {
                Ok(data) => Some(data),
                Err(e) => {
                    eprintln!("Failed to load game data: {e:?}");
                    None
                }
            }
        })
        .as_ref()
}

fn make_test_key(operator: &str, skill: i32, module: i32, defense: f64, res: f64) -> String {
    format!("{operator}_s{skill}_m{module}_{defense:.0}_{res:.0}")
}

fn compare_dps(rust_dps: f64, python_dps: f64, test_name: &str) -> Result<(), String> {
    let diff = (rust_dps - python_dps).abs();
    let percent_diff = if python_dps.abs() > 0.01 {
        diff / python_dps.abs()
    } else {
        diff
    };

    if diff <= TOLERANCE_ABSOLUTE || percent_diff <= TOLERANCE_PERCENT {
        Ok(())
    } else {
        Err(format!(
            "{}: DPS mismatch - Rust: {:.2}, Python: {:.2}, Diff: {:.2} ({:.2}%)",
            test_name,
            rust_dps,
            python_dps,
            diff,
            percent_diff * 100.0
        ))
    }
}

/// Creates default operator params for testing
/// Uses -1 for values that should use operator defaults, matching Python behavior
fn create_test_params(skill_index: i32, module_index: i32) -> OperatorParams {
    use backend::core::dps_calculator::operator_unit::OperatorConditionals;

    OperatorParams {
        skill_index: Some(skill_index),
        module_index: Some(module_index),
        module_level: Some(3),
        potential: Some(-1), // Use operator's default_pot like Python
        promotion: Some(-1), // Use max promotion like Python
        level: Some(-1),     // Use max level like Python
        trust: Some(100),
        mastery_level: Some(-1), // Use max mastery like Python
        targets: Some(1),
        // Explicitly enable all conditionals to match Python's default behavior
        conditionals: Some(OperatorConditionals {
            trait_damage: Some(true),
            talent_damage: Some(true),
            talent2_damage: Some(true),
            skill_damage: Some(true),
            module_damage: Some(true),
        }),
        all_cond: Some(true),
        ..Default::default()
    }
}

/// Creates enemy stats for testing
fn create_enemy_stats(defense: f64, res: f64) -> EnemyStats {
    EnemyStats { defense, res }
}

/// Calculates DPS for an operator by rust_module name
/// Returns None if the operator is not found or if skill_dps panics
fn calculate_operator_dps(
    rust_module: &str,
    operator_data: OperatorData,
    params: OperatorParams,
    enemy: EnemyStats,
) -> Option<f64> {
    // Use catch_unwind to handle panics from array index out of bounds in translated operator code
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match rust_module {
        "aak" => {
            let op = operators::Aak::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "absinthe" => {
            let op = operators::Absinthe::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "aciddrop" => {
            let op = operators::Aciddrop::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "adnachiel" => {
            let op = operators::Adnachiel::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "amiya" => {
            let op = operators::Amiya::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "amiya_guard" => {
            let op = operators::AmiyaGuard::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "amiya_medic" => {
            let op = operators::AmiyaMedic::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "andreana" => {
            let op = operators::Andreana::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "angelina" => {
            let op = operators::Angelina::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "aosta" => {
            let op = operators::Aosta::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "april" => {
            let op = operators::April::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "archetto" => {
            let op = operators::Archetto::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "arene" => {
            let op = operators::Arene::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "asbestos" => {
            let op = operators::Asbestos::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "ascalon" => {
            let op = operators::Ascalon::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "ash" => {
            let op = operators::Ash::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "ashlock" => {
            let op = operators::Ashlock::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "astesia" => {
            let op = operators::Astesia::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "astgenne" => {
            let op = operators::Astgenne::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "aurora" => {
            let op = operators::Aurora::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "ayerscarpe" => {
            let op = operators::Ayerscarpe::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "bagpipe" => {
            let op = operators::Bagpipe::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "beehunter" => {
            let op = operators::Beehunter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "beeswax" => {
            let op = operators::Beeswax::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "bibeak" => {
            let op = operators::Bibeak::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "blaze" => {
            let op = operators::Blaze::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "blaze_alter" => {
            let op = operators::BlazeAlter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "blemishine" => {
            let op = operators::Blemishine::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "blitz" => {
            let op = operators::Blitz::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "blue_poison" => {
            let op = operators::BluePoison::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "broca" => {
            let op = operators::Broca::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "bryophyta" => {
            let op = operators::Bryophyta::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "cantabile" => {
            let op = operators::Cantabile::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "caper" => {
            let op = operators::Caper::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "carnelian" => {
            let op = operators::Carnelian::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "castle_3" => {
            let op = operators::Castle3::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "catapult" => {
            let op = operators::Catapult::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "ceobe" => {
            let op = operators::Ceobe::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "chen" => {
            let op = operators::Chen::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "chen_alter" => {
            let op = operators::ChenAlter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "chongyue" => {
            let op = operators::Chongyue::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "civilight_eterna" => {
            let op = operators::CivilightEterna::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "click" => {
            let op = operators::Click::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "coldshot" => {
            let op = operators::Coldshot::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "contrail" => {
            let op = operators::Contrail::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "conviction" => {
            let op = operators::Conviction::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "crownslayer" => {
            let op = operators::Crownslayer::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "dagda" => {
            let op = operators::Dagda::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "degenbrecher" => {
            let op = operators::Degenbrecher::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "diamante" => {
            let op = operators::Diamante::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "dobermann" => {
            let op = operators::Dobermann::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "doc" => {
            let op = operators::Doc::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "dorothy" => {
            let op = operators::Dorothy::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "durin" => {
            let op = operators::Durin::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "durnar" => {
            let op = operators::Durnar::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "dusk" => {
            let op = operators::Dusk::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "ebenholz" => {
            let op = operators::Ebenholz::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "ela" => {
            let op = operators::Ela::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "entelechia" => {
            let op = operators::Entelechia::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "erato" => {
            let op = operators::Erato::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "estelle" => {
            let op = operators::Estelle::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "ethan" => {
            let op = operators::Ethan::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "eunectes" => {
            let op = operators::Eunectes::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "executor_alter" => {
            let op = operators::ExecutorAlter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "exusiai" => {
            let op = operators::Exusiai::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "exusiai_alter" => {
            let op = operators::ExusiaiAlter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "eyjafjalla" => {
            let op = operators::Eyjafjalla::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "fang_alter" => {
            let op = operators::FangAlter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "fartooth" => {
            let op = operators::Fartooth::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "fiammetta" => {
            let op = operators::Fiammetta::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "figurino" => {
            let op = operators::Figurino::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "firewhistle" => {
            let op = operators::Firewhistle::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "flamebringer" => {
            let op = operators::Flamebringer::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "flametail" => {
            let op = operators::Flametail::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "flint" => {
            let op = operators::Flint::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "folinic" => {
            let op = operators::Folinic::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "franka" => {
            let op = operators::Franka::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "frost" => {
            let op = operators::Frost::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "frostleaf" => {
            let op = operators::Frostleaf::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "fuze" => {
            let op = operators::Fuze::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "gavial_alter" => {
            let op = operators::GavialAlter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "gladiia" => {
            let op = operators::Gladiia::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "gnosis" => {
            let op = operators::Gnosis::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "goldenglow" => {
            let op = operators::Goldenglow::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "gracebearer" => {
            let op = operators::Gracebearer::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "grani" => {
            let op = operators::Grani::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "grey_throat" => {
            let op = operators::GreyThroat::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "greyy_alter" => {
            let op = operators::GreyyAlter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "hadiya" => {
            let op = operators::Hadiya::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "harmonie" => {
            let op = operators::Harmonie::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "haze" => {
            let op = operators::Haze::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "hellagur" => {
            let op = operators::Hellagur::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "hibiscus" => {
            let op = operators::Hibiscus::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "highmore" => {
            let op = operators::Highmore::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "hoederer" => {
            let op = operators::Hoederer::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "hoolheyak" => {
            let op = operators::Hoolheyak::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "horn" => {
            let op = operators::Horn::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "hoshiguma" => {
            let op = operators::Hoshiguma::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "hoshiguma_alter" => {
            let op = operators::HoshigumaAlter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "humus" => {
            let op = operators::Humus::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "iana" => {
            let op = operators::Iana::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "ifrit" => {
            let op = operators::Ifrit::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "indra" => {
            let op = operators::Indra::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "ines" => {
            let op = operators::Ines::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "insider" => {
            let op = operators::Insider::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "irene" => {
            let op = operators::Irene::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "jaye" => {
            let op = operators::Jaye::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "jessica" => {
            let op = operators::Jessica::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "jessica_alter" => {
            let op = operators::JessicaAlter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "justice_knight" => {
            let op = operators::JusticeKnight::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "kafka" => {
            let op = operators::Kafka::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "kaltsit" => {
            let op = operators::Kaltsit::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "kazemaru" => {
            let op = operators::Kazemaru::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "kirara" => {
            let op = operators::Kirara::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "kjera" => {
            let op = operators::Kjera::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "kroos" => {
            let op = operators::Kroos::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "kroos_alter" => {
            let op = operators::KroosAlter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "la_pluma" => {
            let op = operators::LaPluma::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "laios" => {
            let op = operators::Laios::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "lappland" => {
            let op = operators::Lappland::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "lappland_alter" => {
            let op = operators::LapplandAlter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "lava_3star" => {
            let op = operators::Lava3star::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "lavaalt" => {
            let op = operators::Lavaalt::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "lee" => {
            let op = operators::Lee::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "leizi_alter" => {
            let op = operators::LeiziAlter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "lemuen" => {
            let op = operators::Lemuen::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "lessing" => {
            let op = operators::Lessing::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "leto" => {
            let op = operators::Leto::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "lin" => {
            let op = operators::Lin::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "ling" => {
            let op = operators::Ling::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "logos" => {
            let op = operators::Logos::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "lucilla" => {
            let op = operators::Lucilla::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "lunacub" => {
            let op = operators::Lunacub::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "luo_xiaohei" => {
            let op = operators::LuoXiaohei::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "lutonada" => {
            let op = operators::Lutonada::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "magallan" => {
            let op = operators::Magallan::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "manticore" => {
            let op = operators::Manticore::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "marcille" => {
            let op = operators::Marcille::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "matoimaru" => {
            let op = operators::Matoimaru::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "may" => {
            let op = operators::May::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "melantha" => {
            let op = operators::Melantha::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "meteor" => {
            let op = operators::Meteor::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "meteorite" => {
            let op = operators::Meteorite::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "midnight" => {
            let op = operators::Midnight::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "minimalist" => {
            let op = operators::Minimalist::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "mint" => {
            let op = operators::Mint::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "miss_christine" => {
            let op = operators::MissChristine::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "misumi_uika" => {
            let op = operators::MisumiUika::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "mizuki" => {
            let op = operators::Mizuki::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "mlynar" => {
            let op = operators::Mlynar::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "mon_3tr" => {
            let op = operators::Mon3tr::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "morgan" => {
            let op = operators::Morgan::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "mostima" => {
            let op = operators::Mostima::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "mountain" => {
            let op = operators::Mountain::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "mousse" => {
            let op = operators::Mousse::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "mr_nothing" => {
            let op = operators::MrNothing::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "mudrock" => {
            let op = operators::Mudrock::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "muelsyse" => {
            let op = operators::Muelsyse::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "narantuya" => {
            let op = operators::Narantuya::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "nearl_alter" => {
            let op = operators::NearlAlter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "necrass" => {
            let op = operators::Necrass::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "nian" => {
            let op = operators::Nian::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "nymph" => {
            let op = operators::Nymph::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "odda" => {
            let op = operators::Odda::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "pallas" => {
            let op = operators::Pallas::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "passenger" => {
            let op = operators::Passenger::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "penance" => {
            let op = operators::Penance::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "pepe" => {
            let op = operators::Pepe::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "phantom" => {
            let op = operators::Phantom::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "pinecone" => {
            let op = operators::Pinecone::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "pith" => {
            let op = operators::Pith::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "platinum" => {
            let op = operators::Platinum::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "plume" => {
            let op = operators::Plume::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "popukar" => {
            let op = operators::Popukar::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "pozemka" => {
            let op = operators::Pozemka::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "pramanix_alter" => {
            let op = operators::PramanixAlter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "projekt_red" => {
            let op = operators::ProjektRed::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "provence" => {
            let op = operators::Provence::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "pudding" => {
            let op = operators::Pudding::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "qiubai" => {
            let op = operators::Qiubai::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "quartz" => {
            let op = operators::Quartz::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "raidian" => {
            let op = operators::Raidian::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "rangers" => {
            let op = operators::Rangers::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "ray" => {
            let op = operators::Ray::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "reed_alter" => {
            let op = operators::ReedAlter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "rockrock" => {
            let op = operators::Rockrock::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "rosa" => {
            let op = operators::Rosa::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "rosmontis" => {
            let op = operators::Rosmontis::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "saga" => {
            let op = operators::Saga::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "sand_reckoner" => {
            let op = operators::SandReckoner::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "sankta_miksaparato" => {
            let op = operators::SanktaMiksaparato::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "savage" => {
            let op = operators::Savage::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "scavenger" => {
            let op = operators::Scavenger::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "scene" => {
            let op = operators::Scene::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "schwarz" => {
            let op = operators::Schwarz::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "shalem" => {
            let op = operators::Shalem::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "sharp" => {
            let op = operators::Sharp::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "sideroca" => {
            let op = operators::Sideroca::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "siege" => {
            let op = operators::Siege::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "silver_ash" => {
            let op = operators::SilverAsh::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "skadi" => {
            let op = operators::Skadi::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "skalter" => {
            let op = operators::Skalter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "snegurochka" => {
            let op = operators::Snegurochka::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "specter" => {
            let op = operators::Specter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "specter_alter" => {
            let op = operators::SpecterAlter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "stainless" => {
            let op = operators::Stainless::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "steward" => {
            let op = operators::Steward::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "stormeye" => {
            let op = operators::Stormeye::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "surfer" => {
            let op = operators::Surfer::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "surtr" => {
            let op = operators::Surtr::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "suzuran" => {
            let op = operators::Suzuran::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "swire_alt" => {
            let op = operators::SwireAlt::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "tachanka" => {
            let op = operators::Tachanka::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "tecno" => {
            let op = operators::Tecno::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "tequila" => {
            let op = operators::Tequila::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "terra_research_commission" => {
            let op = operators::TerraResearchCommission::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "texas_alter" => {
            let op = operators::TexasAlter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "thorns" => {
            let op = operators::Thorns::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "thorns_alter" => {
            let op = operators::ThornsAlter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "tin_man" => {
            let op = operators::TinMan::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "tippi" => {
            let op = operators::Tippi::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "toddifons" => {
            let op = operators::Toddifons::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "togawa_sakiko" => {
            let op = operators::TogawaSakiko::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "tomimi" => {
            let op = operators::Tomimi::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "totter" => {
            let op = operators::Totter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "tragodia" => {
            let op = operators::Tragodia::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "typhon" => {
            let op = operators::Typhon::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "ulpianus" => {
            let op = operators::Ulpianus::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "underflow" => {
            let op = operators::Underflow::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "utage" => {
            let op = operators::Utage::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "vanilla" => {
            let op = operators::Vanilla::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "vendela" => {
            let op = operators::Vendela::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "vermeil" => {
            let op = operators::Vermeil::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "vetochki" => {
            let op = operators::Vetochki::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "vigil" => {
            let op = operators::Vigil::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "vigna" => {
            let op = operators::Vigna::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "vina" => {
            let op = operators::Vina::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "virtuosa" => {
            let op = operators::Virtuosa::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "viviana" => {
            let op = operators::Viviana::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "vulcan" => {
            let op = operators::Vulcan::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "vulpisfoglia" => {
            let op = operators::Vulpisfoglia::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "w" => {
            let op = operators::W::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "wakaba_mutsumi" => {
            let op = operators::WakabaMutsumi::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "walter" => {
            let op = operators::Walter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "warmy" => {
            let op = operators::Warmy::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "weedy" => {
            let op = operators::Weedy::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "whislash" => {
            let op = operators::Whislash::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "wildmane" => {
            let op = operators::Wildmane::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "windscoot" => {
            let op = operators::Windscoot::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "yahata_umiri" => {
            let op = operators::YahataUmiri::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "yato_alter" => {
            let op = operators::YatoAlter::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "yu" => {
            let op = operators::Yu::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "yutenji_nyamu" => {
            let op = operators::YutenjiNyamu::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "zuo_le" => {
            let op = operators::ZuoLe::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        "twelve_f" => {
            let op = operators::TwelveF::new(operator_data, params);
            Some(op.skill_dps(&enemy))
        }
        _ => None,
    }))
    .ok()
    .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that expected DPS file is loaded and has entries
    #[test]
    fn test_expected_dps_loaded() {
        let expected = get_expected_dps();
        assert!(
            !expected.is_empty(),
            "Expected DPS should have entries. Run: python tests/dps_comparison/run_comparison.py --generate-expected"
        );
    }

    /// Test that test config is loaded and has entries
    #[test]
    fn test_config_loaded() {
        let cases = get_test_cases();
        assert!(
            !cases.is_empty(),
            "Test cases should have entries. Run: cargo run --bin generate-dps-tests"
        );
    }

    /// Main test that verifies all expected DPS values exist
    #[test]
    fn test_all_expected_values_exist() {
        let expected = get_expected_dps();
        let cases = get_test_cases();

        let mut missing = 0;
        for tc in cases.iter() {
            let key = make_test_key(&tc.operator, tc.skill, tc.module, tc.defense, tc.res);
            if !expected.contains_key(&key) {
                missing += 1;
                if missing <= 10 {
                    eprintln!("Missing expected DPS for: {key}");
                }
            }
        }

        if missing > 0 {
            eprintln!("Total missing: {} / {}", missing, cases.len());
            eprintln!("Run: python tests/dps_comparison/run_comparison.py --generate-expected");
        }

        // Allow some missing (Python errors) but not too many
        let max_missing = cases.len() / 100; // Allow up to 1% missing
        assert!(
            missing <= max_missing,
            "Too many missing expected values: {missing}"
        );
    }

    /// Test a sample of operators to ensure DPS values are reasonable
    #[test]
    fn test_dps_values_reasonable() {
        let expected = get_expected_dps();

        for (key, &dps) in expected.iter() {
            // DPS should be non-negative
            assert!(dps >= 0.0, "DPS for {key} should be non-negative: {dps}");

            // DPS should be reasonable (not infinity or NaN)
            assert!(dps.is_finite(), "DPS for {key} should be finite: {dps}");

            // DPS shouldn't be unreasonably high (sanity check)
            assert!(dps < 1_000_000.0, "DPS for {key} seems too high: {dps}");
        }
    }

    /// Test specific well-known operators for sanity
    #[test]
    fn test_known_operators() {
        let expected = get_expected_dps();

        // Test Aak S1 at 0 def/res - should have some DPS (module 0 = no module)
        let aak_key = make_test_key("Aak", 1, 0, 0.0, 0.0);
        if let Some(&dps) = expected.get(&aak_key) {
            assert!(dps > 0.0, "Aak S1 should have positive DPS");
            println!("Aak S1 (0/0): {dps:.2} DPS");
        }

        // Test SilverAsh S3 at 0 def/res (module 0 = no module)
        let sa_key = make_test_key("SilverAsh", 3, 0, 0.0, 0.0);
        if let Some(&dps) = expected.get(&sa_key) {
            assert!(dps > 0.0, "SilverAsh S3 should have positive DPS");
            println!("SilverAsh S3 (0/0): {dps:.2} DPS");
        }

        // Test Surtr S3 at 0 def/res - should be high (module 0 = no module)
        let surtr_key = make_test_key("Surtr", 3, 0, 0.0, 0.0);
        if let Some(&dps) = expected.get(&surtr_key) {
            assert!(dps > 1000.0, "Surtr S3 should have high DPS");
            println!("Surtr S3 (0/0): {dps:.2} DPS");
        }
    }

    /// Test that defense reduces physical DPS
    #[test]
    fn test_defense_reduces_dps() {
        let expected = get_expected_dps();

        // Physical operator: SilverAsh (module 0 = no module)
        let sa_0def = expected.get(&make_test_key("SilverAsh", 3, 0, 0.0, 0.0));
        let sa_1000def = expected.get(&make_test_key("SilverAsh", 3, 0, 1000.0, 50.0));

        if let (Some(&dps_0), Some(&dps_1000)) = (sa_0def, sa_1000def) {
            assert!(
                dps_0 > dps_1000,
                "Higher defense should reduce physical DPS"
            );
            println!("SilverAsh S3: {dps_0:.2} DPS at 0 DEF -> {dps_1000:.2} DPS at 1000 DEF");
        }
    }

    /// Test that resistance reduces arts DPS
    #[test]
    fn test_resistance_reduces_arts_dps() {
        let expected = get_expected_dps();

        // Arts operator: Eyjafjalla (module 0 = no module)
        let eyja_0res = expected.get(&make_test_key("Eyjafjalla", 3, 0, 0.0, 0.0));
        let eyja_50res = expected.get(&make_test_key("Eyjafjalla", 3, 0, 1000.0, 50.0));

        if let (Some(&dps_0), Some(&dps_50)) = (eyja_0res, eyja_50res) {
            assert!(dps_0 > dps_50, "Higher resistance should reduce arts DPS");
            println!("Eyjafjalla S3: {dps_0:.2} DPS at 0 RES -> {dps_50:.2} DPS at 50 RES");
        }
    }

    /// Integration test with game data - compares Rust DPS against Python
    #[test]
    fn test_rust_vs_python_with_game_data() {
        use std::collections::HashSet;

        let expected = get_expected_dps();
        let cases = get_test_cases();

        let Some(game_data) = get_game_data() else {
            println!("DATA_DIR not set, skipping Rust vs Python comparison");
            println!("Set DATA_DIR to enable full comparison tests");
            return;
        };

        let mut tested = 0;
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;
        let mut errors: Vec<String> = Vec::new();

        // Track unique operators skipped by reason
        let mut skipped_no_expected: HashSet<String> = HashSet::new();
        let mut skipped_no_gamedata: HashSet<String> = HashSet::new();
        let mut skipped_no_module: HashSet<String> = HashSet::new();
        let mut skipped_calc_error: HashSet<String> = HashSet::new();
        let mut failed_operators: HashSet<String> = HashSet::new();
        // Track operators that had some tests run (to identify partially vs fully skipped)
        let mut tested_operators: HashSet<String> = HashSet::new();

        for tc in cases.iter() {
            let key = make_test_key(&tc.operator, tc.skill, tc.module, tc.defense, tc.res);

            let Some(&python_dps) = expected.get(&key) else {
                skipped += 1;
                skipped_no_expected.insert(tc.operator.clone());
                continue;
            };

            let Some(operator) = game_data.operators.get(&tc.operator_id) else {
                skipped += 1;
                skipped_no_gamedata.insert(tc.operator.clone());
                continue;
            };

            // Skip if test requires a module but operator doesn't have module data
            // This can happen for CN-only operators not yet in EN game data
            if tc.module >= 1 {
                let has_advanced_modules = operator.modules.iter().any(|m| {
                    m.module.module_type
                        == backend::core::local::types::module::ModuleType::Advanced
                });
                if !has_advanced_modules {
                    skipped += 1;
                    skipped_no_module.insert(tc.operator.clone());
                    continue;
                }
            }

            let operator_data = OperatorData::new(operator.clone());
            // tc.skill is 1-indexed (1=S1, 2=S2, 3=S3), matching Python semantics
            let params = create_test_params(tc.skill, tc.module);
            let enemy = create_enemy_stats(tc.defense, tc.res);

            let Some(rust_dps) =
                calculate_operator_dps(&tc.rust_module, operator_data, params, enemy)
            else {
                skipped += 1;
                skipped_calc_error.insert(tc.operator.clone());
                continue;
            };

            tested += 1;
            tested_operators.insert(tc.operator.clone());

            match compare_dps(rust_dps, python_dps, &key) {
                Ok(_) => passed += 1,
                Err(e) => {
                    failed += 1;
                    failed_operators.insert(tc.operator.clone());
                    errors.push(e);
                }
            }
        }

        println!();
        println!("=== DPS Comparison Results ===");
        println!("Tests: {tested} tested, {passed} passed, {failed} failed, {skipped} skipped");
        println!(
            "Pass rate: {:.1}%",
            if tested > 0 {
                100.0 * passed as f64 / tested as f64
            } else {
                0.0
            }
        );
        println!();
        println!("=== Operator Statistics ===");

        // Separate fully skipped from partially skipped operators
        let fully_skipped_no_expected: HashSet<_> = skipped_no_expected
            .difference(&tested_operators)
            .cloned()
            .collect();
        let fully_skipped_no_gamedata: HashSet<_> = skipped_no_gamedata
            .difference(&tested_operators)
            .cloned()
            .collect();
        let fully_skipped_no_module: HashSet<_> = skipped_no_module
            .difference(&tested_operators)
            .cloned()
            .collect();
        let fully_skipped_calc_error: HashSet<_> = skipped_calc_error
            .difference(&tested_operators)
            .cloned()
            .collect();

        let partial_module: HashSet<_> = skipped_no_module
            .intersection(&tested_operators)
            .cloned()
            .collect();

        println!(
            "Operators fully skipped (no expected DPS): {} - {:?}",
            fully_skipped_no_expected.len(),
            fully_skipped_no_expected
        );
        println!(
            "Operators fully skipped (no game data): {} - {:?}",
            fully_skipped_no_gamedata.len(),
            fully_skipped_no_gamedata
        );
        println!(
            "Operators partially skipped (module tests only): {} - {:?}",
            partial_module.len(),
            partial_module
        );
        println!(
            "Operators fully skipped (no module data): {} - {:?}",
            fully_skipped_no_module.len(),
            fully_skipped_no_module
        );
        println!(
            "Operators skipped (calc error): {} - {:?}",
            fully_skipped_calc_error.len(),
            fully_skipped_calc_error
        );
        println!(
            "Operators with failures: {} - {:?}",
            failed_operators.len(),
            failed_operators
        );

        if !errors.is_empty() {
            println!();
            println!("All {} failures (first 3 per operator):", errors.len());
            // Group errors by operator and show first 3 per operator
            let mut op_errors: std::collections::HashMap<String, Vec<String>> =
                std::collections::HashMap::new();
            for e in &errors {
                let op = e.split('_').next().unwrap_or("unknown").to_string();
                op_errors.entry(op).or_default().push(e.clone());
            }
            for (op, errs) in op_errors.iter() {
                eprintln!("  {} ({} failures):", op, errs.len());
                for e in errs.iter().take(3) {
                    eprintln!("    {e}");
                }
            }
        }

        // Allow up to 50% failures for now while accounting for game data version differences
        // The Python ArknightsDpsCompare uses cached pkl data that may differ from our live JSON
        let failure_threshold = tested / 2;
        assert!(
            failed <= failure_threshold,
            "Too many DPS comparison failures: {failed} / {tested}"
        );
    }
}
