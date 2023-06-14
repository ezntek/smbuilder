use crate::{error::SmbuilderError, prelude::Makeopt};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
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
    pub fn new<P: AsRef<Path>>(spec: Spec, root_dir: P) -> Smbuilder {
        let base_dir_name = if let Some(name) = &spec.name {
            name
        } else {
            &spec.repo.name
        };

        let base_dir = root_dir.as_ref().join(base_dir_name);

        Smbuilder { spec, base_dir }
    }

    fn write_spec(&self) -> Result<(), SmbuilderError> {
        let file_path = self.base_dir.join("smbuilder.yaml");
        let mut smbuilder_specfile = match fs::File::create(&file_path) {
            Ok(f) => f,
            Err(e) => {
                return Err(SmbuilderError::new(
                    Some(Box::new(e)),
                    format!(
                        "failed to create the spec file at {}: ",
                        &file_path.display()
                    ),
                ))
            }
        };

        match smbuilder_specfile.write_all(serde_yaml::to_string(&self.spec).unwrap().as_bytes()) {
            Ok(_) => (),
            Err(e) => {
                return Err(SmbuilderError::new(
                    Some(Box::new(e)),
                    format!(
                        "failed to write the spec into the file at {}: ",
                        &file_path.display()
                    ),
                ))
            }
        };

        Ok(())
    }

    fn clone_repo(&self) -> Result<PathBuf, SmbuilderError> {
        let repo_dir = self.base_dir.join(&self.spec.repo.name);

        Ok(repo_dir) // FIXME: no cloning logic yet
    }

    fn copy_rom<P: AsRef<Path>>(&self, repo_dir: P) -> Result<(), SmbuilderError> {
        match fs::copy(&self.spec.rom.path, repo_dir.as_ref()) {
            Ok(_) => Ok(()),
            Err(e) => Err(SmbuilderError::new(
                Some(Box::new(e)),
                format!(
                    "failed to copy the rom from {} to {}: ",
                    &self.spec.rom.path.display(),
                    repo_dir.as_ref().display(),
                ),
            )),
        }
    }

    fn create_build_script<P: AsRef<Path>>(&self, repo_dir: P) -> Result<(), SmbuilderError> {
        let file_path = self.base_dir.join("build.sh");

        let mut build_script = match fs::File::create(&file_path) {
            Ok(file) => file,
            Err(e) => {
                return Err(SmbuilderError::new(
                    Some(Box::new(e)),
                    format!(
                        "failed to create the build script at {}!",
                        &file_path.display()
                    ),
                ))
            }
        };

        match build_script.write_all(self.spec.get_build_script(repo_dir.as_ref()).as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(SmbuilderError::new(
                Some(Box::new(e)),
                format!(
                    "failed to write to the build script at {}!",
                    &file_path.display()
                ),
            )),
        }
    }

    fn setup_build(&self) {
        // write the spec to disk
        self.write_spec().unwrap();

        // clone the repo
        let repo_dir = self.clone_repo().unwrap();

        // copy the rom
        self.copy_rom(&repo_dir).unwrap();

        // create the build script
        self.create_build_script(&repo_dir).unwrap();
    }

    pub fn build<S>(&self, cmdout_prefix: Option<S>) -> Result<(), SmbuilderError>
    where
        S: AsRef<str> + std::fmt::Display,
    {
        // set the build up first
        self.setup_build();

        // build
        let mut build_cmd = Command::new(&self.base_dir.join("build.sh"));

        let mut spawned_cmd = build_cmd.stdin(Stdio::piped()).spawn();
        let child = match &mut spawned_cmd {
            Ok(c) => c,
            Err(_) => {
                return Err(SmbuilderError::new(
                    None, // FIXME: fix passing the OsError into this
                    "Failed to spawn the build command!",
                ));
            }
        };

        let reader = BufReader::new(child.stdout.take().unwrap());

        for line in reader.lines() {
            let ln = match line {
                Ok(line) => line,
                Err(_) => break, // exit when there is no more output
            };

            if let Some(c) = &cmdout_prefix {
                println!("{}{}", c, ln)
            } else {
                println!("{}", ln)
            }
        }

        match child.wait() {
            Ok(_) => Ok(()),
            Err(e) => Err(SmbuilderError::new(
                Some(Box::new(e)),
                "failed to wait on the build process!",
            )),
        }
    }
}
