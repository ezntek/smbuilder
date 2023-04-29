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

use clap::{Parser, Subcommand, Args, ArgAction, builder::Str};

#[derive(Args, Debug)]
pub struct BuildArgs {
    // some mandatory fields 
    rom_path: String,
    rom_region: String,

    // some options
    #[arg(short, long, default_value_t = 4)]
    jobs: u8,
    #[arg(short, long)]
    name: String,
    #[arg(short = 'T', long)]
    texture_pack: String,
    #[arg(short = 'M', long)]
    makeopts: String,
    #[arg(short, long)]
    repo: String,
    #[arg(long, action = ArgAction::Append)]
    pack: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    NewBuild(BuildArgs),
    EditBuild(BuildArgs),
    Info { name: String },
    Build { name: String },
    Init,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    #[command(subcommand)]
    pub action: Command
}