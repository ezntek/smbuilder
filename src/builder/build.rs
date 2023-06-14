use crate::prelude::Spec;
use crate::{error::SmbuilderError, make_file_executable};
use std::sync::{Arc, Mutex};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use super::{get_needed_setup_tasks, SmbuilderSetupStage};

pub struct Smbuilder {
    spec: Spec,
    base_dir: PathBuf,
}

impl Smbuilder {
    pub fn new<P: AsRef<Path>>(spec: Spec, root_dir: P) -> Smbuilder {
        let base_dir = Smbuilder::create_base_dir(&spec, root_dir).unwrap();
        Smbuilder { spec, base_dir }
    }

    /// this function runs before `new`,
    /// so this will not take in self, but
    /// will return the result that is relevant.
    fn create_base_dir<P: AsRef<Path>>(
        spec: &Spec,
        root_dir: P,
    ) -> Result<PathBuf, SmbuilderError> {
        let base_dir_name = if let Some(name) = &spec.name {
            name
        } else {
            &spec.repo.name
        };

        let unconfirmed_base_dir = root_dir.as_ref().join(base_dir_name);
        let base_dir = if unconfirmed_base_dir.exists() {
            return Ok(unconfirmed_base_dir);
        } else {
            unconfirmed_base_dir
        };

        match fs::create_dir(&base_dir) {
            Ok(_) => Ok(base_dir),
            Err(e) => Err(SmbuilderError::new(
                Some(Box::new(e)),
                format!("failed to create a directory at {:?}", &base_dir),
            )),
        }
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
            Ok(_) => Ok(()),
            Err(e) => Err(SmbuilderError::new(
                Some(Box::new(e)),
                format!(
                    "failed to write the spec into the file at {}: ",
                    &file_path.display()
                ),
            )),
        }
    }

    fn clone_repo(&self) -> Result<PathBuf, SmbuilderError> {
        let repo_name = &self.spec.repo.name;
        let repo_dir = self.base_dir.join(repo_name);

        match git2::build::RepoBuilder::new()
            .branch(&self.spec.repo.branch)
            .clone(&self.spec.repo.url, &repo_dir)
        {
            Ok(_) => Ok(repo_dir),
            Err(e) => Err(SmbuilderError::new(
                Some(Box::new(e)),
                format!(
                    "failed to clone the repository from {} into {}: ",
                    &self.spec.repo.url,
                    &repo_dir.display()
                ),
            )),
        }
    }

    fn copy_rom<P: AsRef<Path>>(&self, repo_dir: P) -> Result<(), SmbuilderError> {
        let rom_copy_target = repo_dir
            .as_ref()
            .join(format!("baserom.{}.z64", &self.spec.rom.region.to_string()));

        match fs::copy(&self.spec.rom.path, rom_copy_target) {
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
            Ok(_) => (),
            Err(e) => {
                return Err(SmbuilderError::new(
                    Some(Box::new(e)),
                    format!(
                        "failed to write to the build script at {}!",
                        &file_path.display()
                    ),
                ))
            }
        };

        make_file_executable(&file_path)
    }

    fn setup_build(&self) {
        use SmbuilderSetupStage::*;

        let needed_targets = get_needed_setup_tasks(&self.spec, &self.base_dir);

        // define some closures for less indents
        let handle_write_spec = || {
            if let Err(e) = self.write_spec() {
                panic!("{}", e)
            }
        };

        let handle_clone_repo = || {
            if let Err(e) = self.clone_repo() {
                panic!("{}", e)
            }
        };

        let handle_copy_rom = |repo_dir: &Path| {
            if let Err(e) = self.copy_rom(repo_dir) {
                panic!("{}", e)
            }
        };

        let handle_create_build_script = |repo_dir: &Path| {
            if let Err(e) = self.create_build_script(repo_dir) {
                panic!("{}", e)
            }
        };

        for target in needed_targets {
            match target {
                WriteSpec => handle_write_spec(),
                CloneRepo => handle_clone_repo(),
                CopyRom => handle_copy_rom(&self.base_dir.join(&self.spec.repo.name)),
                CreateBuildScript => {
                    handle_create_build_script(&self.base_dir.join(&self.spec.repo.name))
                }
            }
        }
    }

    fn symlink_executable<P: AsRef<Path>>(&self, repo_dir: P) -> Result<(), SmbuilderError> {
        let region_str: String = self.spec.rom.region.to_string();
        let orig_path = repo_dir
            .as_ref()
            .join("build")
            .join(format!("{}_pc", &region_str))
            .join(format!("sm64.{}.f3dex2e", &region_str));
        let target_path = repo_dir.as_ref().join("game_executable");

        match symlink(&orig_path, &target_path) {
            Ok(_) => Ok(()),
            Err(e) => Err(SmbuilderError::new(
                Some(Box::new(e)),
                format!(
                    "failed to symlink the executable from {} to {}",
                    orig_path.display(),
                    target_path.display()
                ),
            )),
        }
    }

    pub fn build(&self, cmdout_prefix: Option<String>) -> Result<(), SmbuilderError> {
        // set the build up first
        self.setup_build();

        // build
        let mut build_cmd = Command::new(self.base_dir.join("build.sh"));

        let spawned_cmd = build_cmd.stdin(Stdio::piped()).spawn();
        let child = match spawned_cmd {
            Ok(c) => Arc::new(Mutex::new(c)),
            Err(_) => {
                return Err(SmbuilderError::new(
                    None, // FIXME: fix passing the OsError into this
                    "Failed to spawn the build command!",
                ));
            }
        };

        let child_thread = child.clone();

        std::thread::spawn(move || {
            let mut child = child_thread.lock().unwrap();
            let stdout = child.stdout.take().unwrap();
            let reader = BufReader::new(stdout);

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
        });

        let exit_status = match child.lock().unwrap().wait() {
            Ok(exit_status) => exit_status,
            Err(e) => {
                return Err(SmbuilderError::new(
                    Some(Box::new(e)),
                    "failed to wait on the build process!",
                ))
            }
        };

        let exit_status_code = if let Some(e_code) = exit_status.code() {
            e_code
        } else {
            return Err(SmbuilderError::new(
                None,
                "failed to build the executable: probably terminated by a signal.",
            ));
        };

        if exit_status_code != 0 {
            return Err(SmbuilderError::new(
                None,
                format!(
                    "failed to build the executable with exit code {}",
                    &exit_status_code
                ),
            ));
        }

        self.symlink_executable(self.base_dir.join(&self.spec.repo.name))

        /*
        let reader = BufReader::new(child.stdout.take().unwrap());

        for line in reader.lines() {

        }

        let exit_status = match child.wait() {
            Ok(exit_status) => exit_status,
            Err(e) => {
                return Err(SmbuilderError::new(
                    Some(Box::new(e)),
                    "failed to wait on the build process!",
                ))
            }
        };

        let exit_status_code = if let Some(e_code) = exit_status.code() {
            e_code
        } else {
            return Err(SmbuilderError::new(
                None,
                "failed to build the executable: probably terminated by a signal.",
            ));
        };

        if exit_status_code != 0 {
            return Err(SmbuilderError::new(
                None,
                format!(
                    "failed to build the executable with exit code {}",
                    &exit_status_code
                ),
            ));
        }

        self.symlink_executable(self.base_dir.join(&self.spec.repo.name))
        */
    }
}
