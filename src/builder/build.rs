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

use super::Makeopt;
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

use super::types::Spec;

pub fn get_makeopts_string(makeopts: &[Makeopt]) -> String {
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
        Smbuilder { spec, base_dir }
    }

    pub fn setup_build(&mut self) {
        let mut smbuilder_toml_file = fs::File::create(self.base_dir.join("smbuilder.toml"))
            .expect("creating the smbuilder.toml file failed!");

        match smbuilder_toml_file.write_all(
            serde_yaml::to_string(&self.spec)
                .unwrap() // we'd want to panic if this breaks here anyway ._.
                .as_bytes(),
        ) {
            Ok(_) => (),
            Err(_) => panic!("Failed to write the build specification to the smbuilder.toml!"),
        }

        let repo_dir = &self.base_dir.join(&self.spec.repo.name);

        match git2::build::RepoBuilder::new()
            .branch(&self.spec.repo.branch)
            .clone(&self.spec.repo.url, repo_dir)
        {
            Ok(_) => (),
            Err(_) => panic!(
                "Failed to clone {} into {}!",
                &self.spec.repo.url,
                repo_dir.display()
            ),
        }

        match fs::copy(&self.spec.rom.path, repo_dir) {
            Ok(_) => (),
            Err(_) => panic!(
                "Failed to copy {} into {}!",
                &self.spec.rom.path.display(),
                repo_dir.display()
            ),
        }

        let mut build_script = match fs::File::create(self.base_dir.join("build.sh")) {
            Ok(file) => file,
            Err(_) => panic!(
                "failed to create {}!",
                &self.base_dir.join("build.sh").display()
            ),
        };

        match build_script.write_all(self.spec.get_build_script(repo_dir).as_bytes()) {
            Ok(_) => (),
            Err(_) => panic!(
                "failed to write to the build script at {}!",
                &self.base_dir.join("build.sh").display()
            ),
        }
    }

    pub fn build<S>(&self, cmdout_prefix: Option<S>) -> Result<(), &str>
    where
        S: AsRef<str> + std::fmt::Display,
    {
        let mut build_cmd = Command::new(&self.base_dir.join("build.sh"));

        let child = &mut build_cmd
            .stdin(Stdio::piped())
            .spawn()
            .expect("Failed to spawn the build command!");

        let reader = BufReader::new(child.stdout.take().unwrap());

        for line in reader.lines() {
            let ln = match line {
                Ok(line) => line,
                Err(_) => break,
            };

            if let Some(c) = &cmdout_prefix {
                println!("{}{}", c, ln)
            } else {
                println!("{}", ln)
            }
        }

        match child.wait() {
            Ok(_) => Ok(()),
            Err(_) => panic!("failed to wait on the build process!"),
        }
    }
}
