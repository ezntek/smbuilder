use clap::Parser;

mod cli;

use cli::*;
use smbuilder::{
    prelude::Smbuilder,
    settings::{CmdoutSettings, Settings},
    types::Spec,
};
fn main() {
    let args = cli::Args::parse();

    let Command::Build(build_args) = args.cmd;

    let log_level_setting = if build_args.verbose {
        CmdoutSettings::LogProgress { log_level: 3 }
    } else {
        CmdoutSettings::LogProgress {
            log_level: build_args.log_level,
        }
    };

    let settings = Settings {
        cmdout_settings: log_level_setting,
    };

    let spec = match Spec::from_file(build_args.filename) {
        Ok(s) => s,
        Err(e) => {
            e.pretty_panic(&settings);
            panic!(); // dummy code
        }
    };

    let builder = Smbuilder::new(spec, "./", settings);
    match builder.build() {
        Ok(_) => (),
        Err(e) => e.pretty_panic(&settings),
    };
}
