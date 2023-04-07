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

use std::path::{Path,PathBuf};
use crate::prelude::*;

#[cfg(test)]
mod tests{}


pub struct SmbuilderBuilder<M: MakeoptsType> {
    spec: BuildSpec<M>,
}

impl<M: MakeoptsType> SmbuilderBuilder<M> {
    pub fn new() -> SmbuilderBuilder<M> {
        let default_repo = Repo::default();
        SmbuilderBuilder { 
            spec: BuildSpec {
                jobs: 2,
                name: default_repo.name.clone(),
                additional_makeopts: Vec::new(),
                executable_path: None,
                texture_pack_path: None,
                dynos_packs: Vec::new(),
                repo: default_repo,
                rom: Rom::default(),
            }
        }
    }

    pub fn jobs(mut self, value: u8) -> Self {
        self.spec.jobs = value;
        self
    }

    pub fn name(mut self, value: String) -> Self {
        self.spec.name = value;
        self
    }

    pub fn add_makeopt(mut self, new_makeopt: M) -> Self {
        self.spec.additional_makeopts.push(new_makeopt);
        self
    }

    pub fn append_makeopts(mut self, mut makeopts: Vec<M>) -> Self {
        self.spec.additional_makeopts.append(&mut makeopts);
        self
    }

    pub fn set_makeopts(mut self, makeopts: Vec<M>) -> Self {
        self.spec.additional_makeopts = makeopts;
        self
    }

    pub fn texture_pack_path(mut self, value: String) -> Self {
        match self.spec.repo.supports_textures {
            true => {
                self.spec.texture_pack_path = Some(value);
                return self
            },
            false => self
        }
    }

    pub fn add_dynos_pack(mut self, pack: DynOSPack) -> Self {
        match self.spec.repo.supports_packs {
            true => {
                self.spec.dynos_packs.push(pack);
                self
            },
            false => self
        }
    }

    pub fn append_dynos_packs(mut self, mut packs: Vec<DynOSPack>) -> Self {
        match self.spec.repo.supports_packs {
            true => {
                self.spec.dynos_packs.append(&mut packs);
                self
            },
            false => self
        }
    }

    pub fn set_dynos_packs(mut self, packs: Vec<DynOSPack>) -> Self {
        match self.spec.repo.supports_packs {
            true => {
                self.spec.dynos_packs = packs;
                self
            },
            false => self
        }
    }

    pub fn repo(mut self, value: Repo) -> Self {
        self.spec.repo = value;
        self
    }

    pub fn rom(mut self, value: Rom) -> Self {
        self.spec.rom = value;
        self
    }
}

pub struct Smbuilder<M: MakeoptsType> {
    spec: BuildSpec<M>,
    current_cmd_stdout: Vec<String>, // supposed to be output of a BufReader object .lines() call (so lines from the stdout), too lazy to find type for now
    make_cmd: String, // the actual command
}

impl<M: MakeoptsType> Smbuilder<M> {
    pub fn builder() -> SmbuilderBuilder<M> {
        SmbuilderBuilder::new()
    }

    fn setup(&self) {
        // set up the base directory for easy access later
        let base_dir = Path::new(std::env!("HOME")).join(".local/share/smbuilder");

        // create the build directory
        std::fs::create_dir(&base_dir.join(&self.spec.name)).unwrap();

        // deserialize the build spec
    }
}