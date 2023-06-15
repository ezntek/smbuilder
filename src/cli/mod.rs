use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct BuildArgs {
    pub filename: String,

    #[arg(
        short,
        long,
        help = "choose if the output should be verbose or not (same as --log-level=3)"
    )]
    pub verbose: bool,

    #[arg(
        short = 'Q',
        long,
        help = "choose if there should be no output whatsoever"
    )]
    pub quiet: bool,

    #[arg(
        short = 'L',
        long,
        help = "choose the log level (0 = errors only, 1 = errors + build output, 2 = everything except info, 3 = all/verbose)"
    )]
    pub log_level: u8,
}

#[derive(Parser)]
struct ConfigureArgs {
    rom_region: String,
    rom_path: PathBuf,
    repo_id: u8,
    jobs: Option<u8>,
    custom_name: Option<String>,
    additional_makeopts: Vec<String>,
}

#[derive(Subcommand)]
pub enum Command {
    /// builds a yaml spec.
    Build(BuildArgs),
    // (SUPPOSED TO BE A DOC-COMMENT) configure a brand new spec, and saves it.
    // Configure(ConfigureArgs),

    // TOOD: implement that
}

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Command,
}
