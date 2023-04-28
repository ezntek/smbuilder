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

use std::{path::{Path,PathBuf}, sync::Mutex, fs, io::{Write, BufReader, BufRead}, process::{Stdio, Command, ChildStdout}};
use std::os::unix::fs::PermissionsExt;
use crate::prelude::*;

#[cfg(test)]
mod tests {}

pub struct SmbuilderBuilder<M: MakeoptsType> {
    spec: BuildSpec<M>,
}

impl<M> SmbuilderBuilder<M>
where
    M: MakeoptsType
        + serde::Serialize
        + for<'a> serde::Deserialize<'a>
{
    fn new() -> SmbuilderBuilder<M> {
        let default_repo = Repo::default();
        SmbuilderBuilder { 
            spec: BuildSpec {
                jobs: 2,
                name: default_repo.name.clone(),
                additional_makeopts: Vec::new(),
                executable_path: None,
                texture_pack_path: None,
                dynos_packs: Some(Vec::new()),
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

    pub fn texture_pack_path(mut self, value: PathBuf) -> Self {
        match self.spec.repo.supports_textures {
            true => {
                self.spec.texture_pack_path = Some(value);
                return self
            },
            false => self
        }
    }

    pub fn add_dynos_pack(mut self, pack: DynOSPack) -> Self {
        match &self.spec.repo.supports_packs {
            true => {
                if let Some(ref mut existing_packs) = &mut self.spec.dynos_packs {
                    existing_packs.push(pack);
                } else {
                    self.spec.dynos_packs = Some(vec![pack]);
                }
                self
            },
            false => self
        }
    }

    pub fn append_dynos_packs(mut self, mut packs: Vec<DynOSPack>) -> Self {
        match &self.spec.repo.supports_packs {
            true => {
                if let Some(ref mut existing_packs) = &mut self.spec.dynos_packs {
                    existing_packs.append(&mut packs);
                } else {
                    self.spec.dynos_packs = Some(packs);
                }
                self
            },
            false => self
        }
    }

    pub fn set_dynos_packs(mut self, packs: Vec<DynOSPack>) -> Self {
        match self.spec.repo.supports_packs {
            true => {
                self.spec.dynos_packs = Some(packs);
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

    pub fn build(self) -> Result<Smbuilder<M>, &'static str> {
        match &self.spec.rom == &Rom::default() {
            true => Err("You must supply a baserom in order to compile the project! Please go back and supply a Rom."),
            false => Ok(Smbuilder::new(self.spec)),
        }
    }
}

pub struct Smbuilder<M: MakeoptsType> {
    spec: BuildSpec<M>,
    base_dir: PathBuf,
    cmd_stdout: Option<Mutex<ChildStdout>>,
}

impl<M> Smbuilder<M>
where
    M: MakeoptsType
        + serde::Serialize
        + for<'a> serde::Deserialize<'a>
{
    pub fn builder() -> SmbuilderBuilder<M> {
        SmbuilderBuilder::new()
    } 

    fn new(spec: BuildSpec<M>) -> Smbuilder<M> {
        // set up the base directory for easy access later
        let base_dir = Path::new(std::env!("HOME"))
                                    .join(".local/share/smbuilder")
                                    .join(&spec.name);

        // create the build directory
        fs::create_dir(&base_dir.join(&spec.name)).unwrap();
        
        Smbuilder {
            spec,
            base_dir,
            cmd_stdout: None,
        }
    }

    pub fn setup_build(&mut self) {
        // create the smbuilder.toml
        fs::File::create(&self.base_dir.join("smbuilder.toml"))
            .unwrap()
            .write_all(
                toml::to_string(&self.spec)
                    .unwrap()
                    .as_bytes()
            ).unwrap();
        

        // Create the repo dir
        let repo_dir = &self.base_dir.join(&self.spec.repo.name);

        git2::build::RepoBuilder::new()
            .branch(&self.spec.repo.branch)
            .clone(
                &self.spec.repo.url,
                &repo_dir)
            .unwrap();

        // copy over the baserom
        fs::copy(&self.spec.rom.path, &repo_dir).unwrap();

        // create the build script
        let build_script_string = &self.spec.get_stringified_makeopts(None);
        fs::File::create(&self.base_dir.join("build.sh"))
            .unwrap()
            .write_all(
                build_script_string.as_bytes()
            ).unwrap();
        
        // set the script as executable
        let current_file_perm = fs::metadata(&self.base_dir.join("build.sh"))
            .unwrap()
            .permissions();

        fs::set_permissions(
            &self.base_dir.join("build.sh"),
            fs::Permissions::from_mode(
                current_file_perm.mode()+0o111 // this is a hacky looking version of a chmod +x,
                                               // getting the current mode and adding 0o111 is what chmod +x does.
            )).unwrap();
    }

    pub fn build<S>(&self, cmdout_prefix: S)
    where
        S: AsRef<str> + std::fmt::Display        
    {
        // set things up
        let mut build_cmd = Command::new(&self.base_dir.join("build.sh"));
        
        let child = &mut build_cmd
                                    .stdout(Stdio::piped())
                                    .spawn() // spawn the command
                                    .unwrap();
        
        let reader = BufReader::new(child.stdout.take().unwrap()); // pipe the stdout of the command into a BufReader

        for line in reader.lines() {
            println!("{}{}", cmdout_prefix, line.unwrap()); // print the stdout out from the bufreader (with an optional prefix)
        }

        child.wait().unwrap(); // wait for it to finnish
    }

    pub fn build_silent(&mut self) {
        // set things up
        let mut build_cmd = Command::new(&self.base_dir.join("build.sh"));
        
        let child = &mut build_cmd
                                    .stdout(Stdio::piped())
                                    .spawn() // spawn the command
                                    .unwrap();

        eprintln!("Using the Blocking Builder to build... The process/app will freeze.");
        self.cmd_stdout = Some(Mutex::new(child.stdout.take().unwrap())); // save the stdout to the struct (the function will only write to the stderr)
        
        child.wait().unwrap(); // now wait for it to finnish
    }
}