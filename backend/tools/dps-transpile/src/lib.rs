pub mod expected;
pub mod formulas;
pub mod hand;
pub mod init_state;
pub mod paths;
pub mod transpile;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::Parser;

use paths::{Mode, workspace_root};

#[derive(Debug, Parser)]
#[command(
    name = "dps-transpile",
    about = "Regenerate the DPS/HPS formulas from ArknightsDpsCompare"
)]
pub struct Cli {
    /// Regenerate `operator_formulas.json`
    #[arg(long)]
    pub formulas: bool,
    /// Transpile the DPS formulas to Rust
    #[arg(long)]
    pub transpile: bool,
    /// Regenerate `expected_dps.json` from the Python reference
    #[arg(long)]
    pub expected: bool,
    /// All HPS steps
    #[arg(long)]
    pub healing: bool,
    /// Regenerate `heal_formulas.json`
    #[arg(long)]
    pub healing_formulas: bool,
    /// Transpile the HPS formulas to Rust
    #[arg(long)]
    pub healing_transpile: bool,
    /// Regenerate `expected_hps.json` from the Python reference
    #[arg(long)]
    pub healing_expected: bool,
    /// Print every `__init__` state write and name collision, then exit
    #[arg(long)]
    pub audit: bool,
    /// `ArknightsDpsCompare` checkout; default external/ArknightsDpsCompare
    #[arg(long)]
    pub repo: Option<PathBuf>,
    /// Python 3.10+ with upstream's dependencies; default `[tools] python_bin`
    #[arg(long)]
    pub python: Option<String>,
    /// Game data excel dir for the module check; default from `[assets]` and `[cli]`
    #[arg(long)]
    pub excel: Option<PathBuf>,
}

pub fn run(cli: Cli) -> anyhow::Result<()> {
    let repo_path = cli.repo.clone().map_or_else(paths::default_repo, absolute);
    if cli.audit {
        return init_state::print_formula_audit(&repo_path);
    }
    let config = load_config(&workspace_root().join("config/myrtle.toml"))?;

    let any_dps_flag = cli.formulas || cli.transpile || cli.expected;
    let any_hps_flag =
        cli.healing || cli.healing_formulas || cli.healing_transpile || cli.healing_expected;
    let run_all_dps = !any_dps_flag && !any_hps_flag;

    if run_all_dps || any_dps_flag {
        let py_src = init_state::corrected_source(Mode::Damage, &repo_path)?;
        if run_all_dps || cli.formulas {
            formulas::generate_formulas_json(Mode::Damage, &repo_path, &py_src)?;
        }
        let table = formulas::load_formulas(Mode::Damage)?;
        if run_all_dps || cli.transpile {
            transpile::transpile_all(Mode::Damage, &py_src, &table)?;
        }
        if run_all_dps || cli.expected {
            let excel = cli
                .excel
                .clone()
                .map_or_else(|| excel_dir(&config), absolute);
            expected::generate_expected_dps(
                &repo_path,
                &table,
                &python_bin(&cli, &config),
                &excel,
            )?;
        }
    }

    if any_hps_flag {
        let py_src = init_state::corrected_source(Mode::Healing, &repo_path)?;
        if cli.healing || cli.healing_formulas {
            formulas::generate_formulas_json(Mode::Healing, &repo_path, &py_src)?;
        }
        let table = formulas::load_formulas(Mode::Healing)?;
        if cli.healing || cli.healing_transpile {
            transpile::transpile_all(Mode::Healing, &py_src, &table)?;
        }
        if cli.healing || cli.healing_expected {
            expected::generate_expected_hps(&repo_path, &table, &python_bin(&cli, &config))?;
        }
    }
    Ok(())
}

fn absolute(path: PathBuf) -> PathBuf {
    std::path::absolute(&path).unwrap_or(path)
}

pub fn load_config(path: &Path) -> anyhow::Result<toml::Value> {
    if !path.exists() {
        return Ok(toml::Value::Table(toml::Table::new()));
    }
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    text.parse::<toml::Table>()
        .map(toml::Value::Table)
        .with_context(|| format!("parse {}", path.display()))
}

fn setting<'a>(config: &'a toml::Value, section: &str, key: &str) -> Option<&'a toml::Value> {
    config.get(section)?.get(key)
}

pub fn python_bin(cli: &Cli, config: &toml::Value) -> String {
    cli.python
        .clone()
        .or_else(|| {
            setting(config, "tools", "python_bin")
                .and_then(toml::Value::as_str)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "python3".to_string())
}

/// `<[assets] dir>/<server>/gamedata/excel`, server from `[cli] default_server` then `[server] servers`
pub fn excel_dir(config: &toml::Value) -> PathBuf {
    let assets = setting(config, "assets", "dir")
        .and_then(toml::Value::as_str)
        .unwrap_or("../assets/output");
    let server = setting(config, "cli", "default_server")
        .and_then(toml::Value::as_str)
        .filter(|s| !s.is_empty())
        .or_else(|| {
            setting(config, "server", "servers")
                .and_then(toml::Value::as_array)
                .and_then(|servers| servers.first())
                .and_then(toml::Value::as_str)
        })
        .unwrap_or("en");
    workspace_root()
        .join(assets)
        .join(server)
        .join("gamedata/excel")
}
