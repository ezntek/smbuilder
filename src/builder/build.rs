use colored::Colorize;
use duct::cmd;

use crate::prelude::Spec;
use crate::settings::*;
use crate::{error::SmbuilderError, make_file_executable};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};

use super::{get_needed_setup_tasks, SmbuilderSetupStage};

pub struct Smbuilder {
    spec: Spec,
    base_dir: PathBuf,
    settings: Settings,
    runnable_settings: RunnableSettings,
}

impl Smbuilder {
    pub fn new<P: AsRef<Path>>(spec: Spec, root_dir: P, settings: Settings) -> Smbuilder {
        let runnable_settings = settings.get_runnable();

        let base_dir = match Smbuilder::create_base_dir(&spec, root_dir, &runnable_settings) {
            Ok(p) => p,
            Err(e) => {
                e.pretty_panic(&settings);
                PathBuf::new() // dummy code for da compiler
            }
        };
        Smbuilder {
            spec,
            base_dir,
            settings,
            runnable_settings,
        }
    }

    /// this function runs before `new`,
    /// so this will not take in self, but
    /// will return the result that is relevant.
    fn create_base_dir<P: AsRef<Path>>(
        spec: &Spec,
        root_dir: P,
        runnable_settings: &RunnableSettings,
    ) -> Result<PathBuf, SmbuilderError> {
        let base_dir_name = if let Some(name) = &spec.name {
            name
        } else {
            &spec.repo.name
        };

        runnable_settings.log(format!("creating the base directory at {}", base_dir_name));

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

        self.runnable_settings.log(format!(
            "creating the spec file at {}",
            &file_path.display()
        ));

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

        self.runnable_settings.log(format!(
            "writing the contents of the spec into {}",
            &file_path.display()
        ));

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

        self.runnable_settings.log("cloning the repository");

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

        self.runnable_settings
            .log("copying the baserom into the correct location...");

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
                e.pretty_panic(&self.settings)
            }
        };

        let handle_clone_repo = || {
            if let Err(e) = self.clone_repo() {
                e.pretty_panic(&self.settings)
            }
        };

        let handle_copy_rom = |repo_dir: &Path| {
            if let Err(e) = self.copy_rom(repo_dir) {
                e.pretty_panic(&self.settings)
            }
        };

        let handle_create_build_script = |repo_dir: &Path| {
            if let Err(e) = self.create_build_script(repo_dir) {
                e.pretty_panic(&self.settings)
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

    pub fn build(&self) -> Result<(), SmbuilderError> {
        // set the build up first
        self.setup_build();

        // build
        let build_cmdout = cmd!(self.base_dir.join("build.sh")).stderr_to_stdout();

        let output = build_cmdout.reader().unwrap(); // FIXME: unwrap
        let reader = BufReader::new(output);

        for line in reader.lines() {
            let ln = match line {
                Ok(line) => line,
                Err(e) => {
                    return Err(SmbuilderError::new(
                        Some(Box::new(e)),
                        "the build command failed to run",
                    ))
                } // exit when there is no more output
            };

            println!("{}{}", "make: ".bold().blue(), ln)
        }

        self.symlink_executable(self.base_dir.join(&self.spec.repo.name))
    }
}
