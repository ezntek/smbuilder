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

use std::{path::PathBuf, fs, io::{Write, BufReader, BufRead}, process::{Command, Stdio}};
use crate::Makeopt;
use super::types::Spec;
use colored::Colorize;

pub fn get_makeopts_string(makeopts: &Vec<Makeopt>) -> String {
    let mut retval = String::from("");

    for opt in makeopts.iter() {
        retval.push_str(format!("{}={} ", opt.key, opt.value).as_str());
    }

    retval
}

pub struct Smbuilder {
    spec: Spec,
    base_dir: PathBuf,
}

impl Smbuilder {
    pub fn new(spec: Spec, base_dir: PathBuf) -> Smbuilder {
        Smbuilder {
            spec, base_dir
        }
    }

    pub fn setup_build(&mut self) {
        let mut smbuilder_toml_file = fs::File::create(&self.base_dir.join("smbuilder.toml"))
            .expect("creating the smbuilder.toml file failed!");

        smbuilder_toml_file.write_all(
            toml::to_string(&self.spec)
                .expect("Failed to parse the `Spec` into a toml string!")
                .as_bytes()
        ).expect("Failed to write the build specification to the smbuilder.toml!");

        let repo_dir = &self.base_dir.join(&self.spec.repo.name);
        
        git2::build::RepoBuilder::new()
            .branch(&self.spec.repo.branch)
            .clone(
                &self.spec.repo.url,
                &repo_dir)
            .expect(format!("Failed to clone {} into {}!", &self.spec.repo.url, repo_dir.display()).as_str());

        fs::copy(&self.spec.rom.path, &repo_dir)
            .expect(format!("Failed to copy {} into {}!", &self.spec.rom.path.display(), repo_dir.display()).as_str());
                
        fs::File::create(&self.base_dir.join("build.sh"))   
            .expect(format!("failed to create {}!", &self.base_dir.join("build.sh").display()).as_str())
            .write_all(
                &self.spec.get_build_script(repo_dir).as_bytes()
            ).expect(format!("Failed to write {}!", &self.base_dir.join("build.sh").display()).as_str());
    }

    pub fn build<S>(&self, cmdout_prefix: Option<S>)
    where
        S: AsRef<str>
         + std::fmt::Display
    {
        let mut build_cmd = Command::new(&self.base_dir.join("build.sh"));

        let child = &mut build_cmd
                                    .stdin(Stdio::piped())
                                    .spawn()
                                    .expect("Failed to spawn the build command!");
        
        let reader = BufReader::new(child.stdout.take().unwrap());

        for line in reader.lines() {
            if let Some(c) = &cmdout_prefix {
                println!("{}{}", c, line.unwrap())
            } else {
                println!("{}", line.unwrap())
            }
        }

        child.wait().unwrap();
    }
}