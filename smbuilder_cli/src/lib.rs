// Copyright 2023 Eason Qin <eason@ezntek.com>.
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
pub mod cli_parser;

use cli_parser::*;
use smbuilder::prelude::*;

#[allow(unused_imports)] // XXX: we're gonna need it for the macros...
use colored::Colorize;

#[cfg(test)]
mod tests {}

// nice macros for logging

#[macro_export]
macro_rules! log_info {
    ( $text:expr ) => {
        println!("{}{}", "Info: ".bold().cyan(), $text)
    };
}

#[macro_export]
macro_rules! log_warn {
    ( $text:expr ) => {
        println!("{}{}", "Info: ".bold().yellow(), $text)
    };
}

#[macro_export]
macro_rules! log_err {
    ( $text:expr ) => {
        println!("{}{}", "Info: ".bold.red(), $text)
    };
}

enum MakeoptsTypes {
    Render96ex(Render96exMakeopts),
    Sm64ex(Sm64exMakeopts),
    Sm64exCoop(Sm64exCoopMakeopts)
}

fn new_build(args: BuildArgs, repo: Repo) {
    let builder: SmbuilderBuilder<_> = Smbuilder::builder()
}

pub fn run(args: CliArgs) {
    match args.action {
        Command::Init => smbuilder::init(),
        Command::Info { name } => log_info!("The feature \"Info\" is yet to be implemented. Called Info {{{}}}", name),
        Command::Build { name } => todo!(),
        Command::EditBuild(args) => todo!(),
        Command::NewBuild(args) => todo!(),
    }
}