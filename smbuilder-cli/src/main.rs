use std::path::PathBuf;

use clap::Parser;
use colored::Colorize;
use smbuilder::prelude::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    base_dir: PathBuf,
}

fn main() {
    color_eyre::install().unwrap();

    let args = Args::parse();

    if !args.base_dir.is_dir() {
        panic!("{} is not a directory! please enter the path to a directory with an `smbuilder.yaml` in the root of it.", args.base_dir.display());
    }

    let mut callbacks = Callbacks::empty()
        .log(|log_type, text| {
            use LogType::*;

            match log_type {
                Error => println!("{}{}", "error: ".bold().red(), text),
                Warn => println!("{}{}", "warn: ".bold().magenta(), text),
                BuildOutput => println!("{}{}", "make: ".bold().cyan(), text),
                Info => println!("{}{}", "info: ".bold().blue(), text),
            };
        })
        .repo_clone_progress(|recv_objs, total_objs, bytes_transferred| {
            print!(
                "{} {}/{} ({}%) objects transferred ({} KiB transferred)\r",
                "clone:".bold().green(),
                recv_objs,
                total_objs,
                (recv_objs * 100) / total_objs,
                (bytes_transferred as f64 / 1024_f64).floor(),
            )
        })
        .new_setup_stage(|stage| {
            println!("{}{}", "stage: ".bold().green(), stage.to_string());
        });

    let spec_path = args.base_dir.join("smbuilder.yaml");

    let spec = Spec::from_file_checked(&spec_path, &mut callbacks)
        .unwrap_or_else(|e| panic!("failed to create the spec: {}", e));

    let mut builder = Builder::new(spec, &args.base_dir, callbacks)
        .unwrap_or_else(|e| panic!("failed to create the builder: {}", e));

    builder.build();
}
