use backend::dps::engine;
use backend::dps::operator_unit::{EnemyStats, OperatorParams};
use dotenv::dotenv;
use std::path::Path;
use std::time::Instant;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // ── Game data loading ───────────────────────────────────────────────
    let data_dir = Path::new("../assets/output/gamedata/excel");
    let assets_dir = Path::new("../assets/output");

    println!("Loading game data...");
    let start = Instant::now();

    let data = match backend::core::gamedata::init_game_data(data_dir, assets_dir) {
        Ok(data) => {
            let elapsed = start.elapsed();
            println!("Loaded in {elapsed:.2?}");
            println!("  Operators: {}", data.operators.len());
            println!("  Skills: {}", data.skills.len());
            println!("  Modules: {}", data.modules.equip_dict.len());
            println!("  Skins: {}", data.skins.char_skins.len());
            println!("  Voices: {}", data.voices.char_words.len());
            println!("  Enemies: {}", data.enemies.enemy_data.len());
            println!("  Stages: {}", data.stages.len());
            println!("  Zones: {}", data.zones.len());
            println!("  Medals: {}", data.medals.medals.len());
            println!("  Chibis: {}", data.chibis.characters.len());
            data
        }
        Err(e) => {
            eprintln!("Failed to load game data: {e}");
            std::process::exit(1);
        }
    };

    if let Some(operator) = data.operators.get("char_211_adnach") {
        let params = OperatorParams::default();
        let enemy = EnemyStats {
            defense: 200.0,
            res: 0.0,
        };

        match engine::calculate_dps(operator, params, &enemy) {
            Some(result) => {
                println!("\nDPS Test: Adnachiel vs 200 DEF");
                println!("  Skill DPS: {:.2}", result.skill_dps);
                println!("  Total Damage: {:.2}", result.total_damage);
                println!("  Average DPS: {:.2}", result.average_dps);
            }
            None => eprintln!("  DPS calc failed for Adnachiel"),
        }
    }
}
