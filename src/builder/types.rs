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

use super::{build::get_makeopts_string, make_file_executable};
use crate::*;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(Default, Builder, Debug, Deserialize, Serialize)]
#[builder(setter(into))]
pub struct Spec {
    pub rom: Rom,
    pub repo: Repo,
    pub jobs: u8,
    pub name: String,
    pub additional_makeopts: Vec<Makeopt>,
    pub packs: Option<Vec<Datapack>>,
}

impl Spec {
    pub fn from_file(path: PathBuf) -> Result<Spec, String> {
        let file_string = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => return Err(format!("Failed to read {}: {}", &path.display(), e)),
        };

        match toml::from_str(&file_string) {
            Ok(s) => s,
            Err(e) => Err(format!(
                "Failed to parse {} into a toml: {}",
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
