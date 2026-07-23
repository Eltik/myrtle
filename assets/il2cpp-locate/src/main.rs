//! CLI for `il2cpp-locate`: rank dynchar entrance camera-follow candidates in a
//! `dump.cs` produced by Il2CppDumper.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use il2cpp_locate::{Candidate, rank};

#[derive(Parser)]
#[command(
    name = "il2cpp-locate",
    about = "Rank camera-follow method candidates in an Il2CppDumper dump.cs \
             (dynchar entrance-camera recovery)"
)]
struct Cli {
    /// Path to the Il2CppDumper `dump.cs` file.
    #[arg(short, long)]
    input: PathBuf,

    /// Emit JSON instead of a human-readable table.
    #[arg(long)]
    json: bool,

    /// Show at most this many candidates (0 = all).
    #[arg(short = 'n', long, default_value = "25")]
    limit: usize,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let text = std::fs::read_to_string(&cli.input)
        .with_context(|| format!("reading {}", cli.input.display()))?;

    let mut candidates = rank(&text);
    if cli.limit > 0 && candidates.len() > cli.limit {
        candidates.truncate(cli.limit);
    }

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&candidates)?);
    } else {
        print_table(&candidates);
    }
    Ok(())
}

fn print_table(candidates: &[Candidate]) {
    if candidates.is_empty() {
        println!("No camera-follow candidates found. Check the dump.cs path/format.");
        return;
    }

    println!(
        "{:>2}  {:>5}  {:<24}  {:<28}  {:<12}  {:<12}",
        "#", "score", "namespace", "type", "offset", "va"
    );
    println!("{}", "-".repeat(96));
    for (i, c) in candidates.iter().enumerate() {
        let offset = c
            .method
            .offset
            .map_or_else(|| "-".to_string(), |v| format!("{v:#x}"));
        let va = c
            .method
            .va
            .map_or_else(|| "-".to_string(), |v| format!("{v:#x}"));
        println!(
            "{:>2}  {:>5}  {:<24}  {:<28}  {:<12}  {:<12}",
            i + 1,
            c.score,
            truncate(&c.method.namespace, 24),
            truncate(&c.method.type_name, 28),
            offset,
            va
        );
    }

    // Detail block for the top few.
    println!("\n=== top candidates ===");
    for (i, c) in candidates.iter().take(5).enumerate() {
        println!("\n#{}  score={}", i + 1, c.score);
        println!("  {}::{}", c.method.namespace, c.method.type_name);
        println!("  {}", c.method.signature);
        println!("  reasons: {}", c.reasons.join(", "));
        for line in c.nav_lines() {
            println!("  {line}");
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let keep: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{keep}…")
    }
}
