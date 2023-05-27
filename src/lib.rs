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

#[allow(unused_imports)]
#[macro_use]
extern crate derive_builder;

pub mod builder;
pub mod ui;

#[allow(unused_imports)] // used in a macro
use colored::Colorize;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum Region {
    #[default]
    US,
    EU,
    JP,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Rom {
    pub region: Region,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Repo {
    pub name: String,
    pub url: String,
    pub branch: String,
    pub supports_packs: bool,
    pub supports_textures: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Makeopt {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Datapack {
    pub label: String,
    pub path: PathBuf,
    pub enabled: bool,
}

#[macro_export]
macro_rules! log_err {
    ($text:literal) => {
        println!("{}{}", "Err: ".bold().red().as_str(), $text);
    };
}

#[macro_export]
macro_rules! log_warn {
    ($text:literal) => {
        println!("{}{}", "Warn: ".bold().yellow().as_str(), $text);
    };
}

#[macro_export]
macro_rules! log_info {
    ($text:literal) => {
        println!("{}{}", "Err: ".bold().blue().as_str(), $text);
    };
}
