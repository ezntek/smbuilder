use std::path::PathBuf;

use clap::Parser;
use colored::Colorize;
use smbuilder::prelude::*;

#[derive(Parser)]
struct Args {
    file: PathBuf,
}

fn main() {
    let args = Args::parse();

    let spec = Spec::from_file(args.file).unwrap();

    let builder = Smbuilder::new(spec, PathBuf::from("./"));

    builder
        .build(Some("make: ".to_string().bold().blue().to_string()))
        .unwrap();
}
