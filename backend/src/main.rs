use std::path::Path;
use std::time::Instant;

use backend::database::init;

#[tokio::main]
async fn main() {
    let data_dir = Path::new("../assets/output/gamedata/excel");
    let assets_dir = Path::new("../assets/output");

    println!("Loading game data...");
    let start = Instant::now();

    match backend::core::gamedata::init_game_data(data_dir, assets_dir) {
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
        }
        Err(e) => {
            eprintln!("Failed to load game data: {e}");
            std::process::exit(1);
        }
    }

    let _pool = init("postgresql://postgres:password@localhost:5432").await;
}
