use crate::prelude::{Smbuilder, Spec};
use crate::settings::RunnableSettings;
use crate::{error::Error, make_file_executable};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub struct SmbuilderWrapper {
    pub spec: Spec,
    pub base_dir: PathBuf,
    pub runnable_settings: Box<dyn RunnableSettings>,
    builder: Box<dyn Smbuilder>,
}

impl SmbuilderWrapper {
    pub fn new<P: AsRef<Path>>(
        spec: Spec,
        root_dir: P,
        runnable_settings: Box<dyn RunnableSettings>,
        builder: Box<dyn Smbuilder>,
    ) -> Result<SmbuilderWrapper, Error> {
        let base_dir = SmbuilderWrapper::create_base_dir(&spec, root_dir, &runnable_settings)?;

        let result = SmbuilderWrapper {
            spec,
            base_dir,
            runnable_settings,
            builder,
        };

        Ok(result)
    }

    /// this function runs before `new`,
    /// so this will not take in self, but
    /// will return the result that is relevant.
    fn create_base_dir<P: AsRef<Path>>(
        spec: &Spec,
        root_dir: P,
        runnable_settings: &Box<dyn RunnableSettings>,
    ) -> Result<PathBuf, Error> {
        let base_dir_name = if let Some(name) = &spec.name {
            name
        } else {
            &spec.repo.name
        };

        (*runnable_settings).log_info(&format!("creating the base directory at {}", base_dir_name));

        let unconfirmed_base_dir = root_dir.as_ref().join(base_dir_name);
        let base_dir = if unconfirmed_base_dir.exists() {
            return Ok(unconfirmed_base_dir);
        } else {
            unconfirmed_base_dir
        };

        match fs::create_dir(&base_dir) {
            Ok(_) => Ok(base_dir),
            Err(e) => Err(Error::new(
                Some(Box::new(e)),
                format!("failed to create a directory at {:?}", &base_dir),
            )),
        }
    }

    pub fn write_spec(&self) -> Result<(), Error> {
        let file_path = self.base_dir.join("smbuilder.yaml");

        (*self.runnable_settings).log_info(&format!(
            "creating the spec file at {}",
            &file_path.display()
        ));

        let mut smbuilder_specfile = match fs::File::create(&file_path) {
            Ok(f) => f,
            Err(e) => {
                return Err(Error::new(
                    Some(Box::new(e)),
                    format!(
                        "failed to create the spec file at {}: ",
                        &file_path.display()
                    ),
                ))
            }
        };

        (*self.runnable_settings).log_info(&format!(
            "writing the contents of the spec into {}",
            &file_path.display()
        ));

        match smbuilder_specfile.write_all(serde_yaml::to_string(&self.spec).unwrap().as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::new(
                Some(Box::new(e)),
                format!(
                    "failed to write the spec into the file at {}: ",
                    &file_path.display()
                ),
            )),
        }
    }

    pub fn clone_repo(&self) -> Result<PathBuf, Error> {
        let repo_name = &self.spec.repo.name;
        let repo_dir = self.base_dir.join(repo_name);

        (*self.runnable_settings).log_info(&"cloning the repository".to_string());

        match git2::build::RepoBuilder::new()
            .branch(&self.spec.repo.branch)
            .clone(&self.spec.repo.url, &repo_dir)
        {
            Ok(_) => Ok(repo_dir),
            Err(e) => Err(Error::new(
                Some(Box::new(e)),
                format!(
                    "failed to clone the repository from {} into {}: ",
                    &self.spec.repo.url,
                    &repo_dir.display()
                ),
            )),
        }
    }

    pub fn copy_rom<P: AsRef<Path>>(&self, repo_dir: P) -> Result<(), Error> {
        let rom_copy_target = repo_dir
            .as_ref()
            .join(format!("baserom.{}.z64", &self.spec.rom.region.to_string()));

        (*self.runnable_settings)
            .log_info(&"copying the baserom into the correct location".to_string());

        match fs::copy(&self.spec.rom.path, rom_copy_target) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::new(
                Some(Box::new(e)),
                format!(
                    "failed to copy the rom from {} to {}: ",
                    &self.spec.rom.path.display(),
                    repo_dir.as_ref().display(),
                ),
            )),
        }
    }

    pub fn create_build_script<P: AsRef<Path>>(&self, repo_dir: P) -> Result<(), Error> {
        let file_path = self.base_dir.join("build.sh");

        let mut build_script = match fs::File::create(&file_path) {
            Ok(file) => file,
            Err(e) => {
                return Err(Error::new(
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
                return Err(Error::new(
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
        (*self.builder).setup_build(&self);
    }

    pub fn build(&self) -> Result<(), Error> {
        // set the build up first
        self.setup_build();

        // build
        (*self.builder).build(&self)
    }
}
