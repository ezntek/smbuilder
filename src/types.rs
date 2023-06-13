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

use crate::prelude::{get_makeopts_string, make_file_executable};
use derive_builder::Builder;
use std::fmt::Debug;
use std::fs;
use std::io::Write;
use std::path::Path;

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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TexturePack {
    pub path: PathBuf,
    pub enabled: bool,
}

#[derive(Default, Builder, Debug, Deserialize, Serialize)]
#[builder(setter(into))]
pub struct Spec {
    pub rom: Rom,
    pub repo: Repo,
    pub jobs: u8,
    pub name: String,
    pub additional_makeopts: Vec<Makeopt>,
    pub texture_pack: TexturePack,
    pub packs: Vec<Datapack>,
}

impl Spec {
    pub fn from_file(path: PathBuf) -> Result<Spec, String> {
        let file_string = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => return Err(format!("Failed to read {}: {}", &path.display(), e)),
        };

        match serde_yaml::from_str(&file_string) {
            Ok(s) => s,
            Err(e) => Err(format!(
                "Failed to parse {} into a yaml: {}",
                &path.display(),
                e
            )),
        }
    }

    pub fn get_build_script(&self, repo_path: &Path) -> String {
        format!(
            "
#!/bin/sh

echo \"Script Generated by smbuilder.\"
echo \"DO NOT EDIT; YOUR CHANGES WILL NOT\"
echo \"BE SAVED.\"

make -C {} {} -j{}
        ",
            repo_path.display(),
            get_makeopts_string(&self.additional_makeopts),
            self.jobs
        )
    }

    pub fn write_build_script(&self, repo_path: &Path) -> Result<(), String> {
        let script = self.get_build_script(repo_path);

        let base_path = match repo_path.parent() {
            Some(base_path) => Ok(base_path.to_path_buf()),
            None => Err("the repository path somehow has no parent directory!"), // early return #1
        };

        let build_script_path = match base_path {
            Ok(path) => path,
            Err(e) => return Err(e.to_string()), // return whatever I wrote above
        };

        let mut build_script_file = match fs::File::create(&build_script_path) {
            Ok(file) => file,
            Err(e) => {
                return Err(format!(
                    "failed to create file at {}: {}",
                    &build_script_path.display(),
                    e
                ))
            } // early return #2
        };

        match build_script_file.write(script.as_bytes()) {
            Ok(_) => (),
            Err(e) => {
                return Err(format!(
                    "failed to write file at {}: {}",
                    &build_script_path.display(),
                    e
                ))
            } // early return #3
        };

        match make_file_executable(&build_script_path) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
