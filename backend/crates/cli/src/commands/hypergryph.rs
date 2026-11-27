use clap::Subcommand;
use myrtle_hypergryph::DeviceIds;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Print fresh device ids as TOML
    DeviceIds,
}

pub fn run(command: Command) -> anyhow::Result<()> {
    match command {
        Command::DeviceIds => print!("{}", fragment(&DeviceIds::generate())),
    }
    Ok(())
}

#[must_use]
pub fn fragment(ids: &DeviceIds) -> String {
    format!(
        "[hypergryph]\ndevice_id = \"{}\"\ndevice_id2 = \"{}\"\ndevice_id3 = \"{}\"\n",
        ids.device_id, ids.device_id2, ids.device_id3
    )
}
