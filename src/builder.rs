//! The API and rust representation(s) of core build processes that are involved in building a port.

use crate::logger::Log;
use crate::prelude::Region;
use crate::SmbuilderError;
use crate::{make_file_executable, prelude::Spec};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

/// A trait to represent a builder struct.
///
/// **Note:** Structs that implements this trait
/// will be wrapped by and will take
/// in a `BuildWrapper`.
///
/// Implementors must implement custom logic for
/// handling the possilbe errors from the provided
/// setup functions in the wrapper and their
/// implementation of running the build script generated
/// by `create_build_script`.
///
/// TODO: example
///
/// implementors must do this as the different
/// use cases for this crate will need different
/// ways of handling errors that are thrown
/// from core functions like these. As an example,
/// a GUI may need to spawn an error dialog, but a
/// CLI may just panic, etc.
///
pub trait Smbuilder {
    /// The build setup function.
    ///
    /// This is where one implements custom logic
    /// to handle `SmbuilderError`s that may be
    /// returned from the wrapper's given setup functions.
    ///
    /// TODO: example
    ///
    /// TODO: further explaination
    fn setup_build(&self, wrapper: &BuildWrapper);

    /// The build function.
    ///
    /// This is where one implements custom logic
    /// behind running the build script provided by
    /// `create_build_script` from the wrapper class.
    ///
    /// TODO: example
    ///
    /// This architecture allows for different ways
    /// of capturing the standard output from the
    /// build command in the script, handling exit codes,
    /// etc.
    fn build(&self, wrapper: &BuildWrapper) -> Result<(), SmbuilderError>;
}

#[derive(Debug)]
/// An enum to represent the different "setup stages"
/// involved in building a port.
///
/// It includes critical steps to be able to
/// build a basic, vanilla port.
pub enum SmbuilderSetupStage {
    /// Write the spec file to disk.
    WriteSpec,

    /// Clone the repository
    /// as per the info in the spec.
    CloneRepo,

    /// Copy the base ROM (to extract
    /// the assets from) from the
    /// specified location in the spec
    /// to the correct location for the build.
    CopyRom,

    /// Create the build script, containing
    /// a small message and the (possibly long)
    /// command to compile the port, after
    /// it has been "prepared" (set up).
    CreateBuildScript,
}

impl ToString for SmbuilderSetupStage {
    fn to_string(&self) -> String {
        use SmbuilderSetupStage::*;

        let result = match self {
            WriteSpec => "write the spec to disk",
            CloneRepo => "clone the repository",
            CopyRom => "copy the base ROM",
            CreateBuildScript => "create the build script",
        };

        result.to_owned()
    }
}

/// The wrapper struct for an `Smbuilder` implementor.
///
/// Includes fields and methods that makes up the core of
/// any smbuilder-derived app.
pub struct BuildWrapper {
    /// The spec that the build will be built with.
    pub spec: Spec,

    /// The base directory, the dir where the spec lives
    pub base_dir: PathBuf,

    /// The logger.
    pub logger: Box<dyn Log>,
    builder: Box<dyn Smbuilder>,
}

impl BuildWrapper {
    /// Creates a new `BuildWrapper`.
    ///
    /// It creates the base directory from
    /// the parameter `root_dir`, which **is not**
    /// the root directory of your disk, rather
    /// it is the parent directory of where the
    /// base directory will be. Useful if one
    /// chooses to put it into a location such as
    /// `$HOME/.local/share`, for frontends.
    ///
    /// It takes in a runnable settings instance
    /// for actions such as logging.
    ///
    pub fn new<P: AsRef<Path>>(
        spec: Spec,
        root_dir: P,
        runnable_settings: Box<dyn Log>,
        builder: Box<dyn Smbuilder>,
    ) -> Result<BuildWrapper, SmbuilderError> {
        let base_dir = BuildWrapper::create_base_dir(&spec, root_dir, &*runnable_settings)?;

        let result = BuildWrapper {
            spec,
            base_dir,
            logger: runnable_settings,
            builder,
        };

        Ok(result)
    }

    fn create_base_dir<P: AsRef<Path>>(
        spec: &Spec,
        root_dir: P,
        runnable_settings: &dyn Log,
    ) -> Result<PathBuf, SmbuilderError> {
        // this function runs before `new`,
        // so this will not take in self, but
        // will return the result that is relevant.

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
            Err(e) => Err(SmbuilderError::new(
                Some(Box::new(e)),
                format!("failed to create a directory at {:?}", &base_dir),
            )),
        }
    }

    /// Writes the current spec to disk.
    pub fn write_spec(&self) -> Result<(), SmbuilderError> {
        let file_path = self.base_dir.join("smbuilder.yaml");

        (*self.logger).log_info(&format!(
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

        (*self.logger).log_info(&format!(
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

    /// Clones the repository as specified in the
    /// spec to disk.
    pub fn clone_repo(&self) -> Result<PathBuf, SmbuilderError> {
        let repo_name = &self.spec.repo.name;
        let repo_dir = self.base_dir.join(repo_name);

        (*self.logger).log_info("cloning the repository");

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

    /// Copies the ROM from the path specified in the spec
    /// to the path needed for the build to succeed.
    pub fn copy_rom<P: AsRef<Path>>(&self, repo_dir: P) -> Result<(), SmbuilderError> {
        let rom_copy_target = repo_dir
            .as_ref()
            .join(format!("baserom.{}.z64", &self.spec.rom.region.to_string()));

        (*self.logger).log_info("copying the baserom into the correct location");

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

    /// Creates the build script in the `base_dir`.
    pub fn create_build_script<P: AsRef<Path>>(&self, repo_dir: P) -> Result<(), SmbuilderError> {
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
        (*self.builder).setup_build(self);
    }

    /// Wrapper function for the inner trait's build function.
    pub fn build(&self) -> Result<(), SmbuilderError> {
        // set the build up first
        self.setup_build();

        // build
        (*self.builder).build(self)
    }
}

/// Get the core setup tasks that are needed.
///
/// Returns a list of `SmbuilderSetupStage`.
pub fn get_needed_setup_tasks<P: AsRef<Path>>(
    spec: &Spec,
    base_dir: P,
    runnable_settings: Box<dyn Log>,
) -> Vec<SmbuilderSetupStage> {
    use SmbuilderSetupStage::*;

    let base_dir = base_dir.as_ref();
    let mut needed_stages: Vec<SmbuilderSetupStage> = Vec::new();

    // check if the spec exists in the dir
    if !base_dir.join("smbuilder.yaml").exists() {
        needed_stages.push(WriteSpec)
    }

    // check if the repo is cloned
    if !base_dir.join(&spec.repo.name).exists() {
        needed_stages.push(CloneRepo)
    }

    // check if the rom exists
    if !base_dir
        .join(&spec.repo.name)
        .join(format!("baserom.{}.z64", spec.rom.region.to_string()))
        .exists()
    {
        needed_stages.push(CopyRom)
    }

    // check if the build script exists
    if !base_dir.join("build.sh").exists() {
        needed_stages.push(CreateBuildScript)
    }

    // log
    let needed_stages_string = needed_stages
        .iter()
        .map(|elem| elem.to_string())
        .collect::<Vec<String>>()
        .join(", ");

    (*runnable_settings).log_info(&format!("needed tasks: {}", needed_stages_string));

    // return
    needed_stages
}

impl ToString for Region {
    fn to_string(&self) -> String {
        use Region::*;

        let retval = match self {
            US => "us",
            EU => "eu",
            JP => "jp",
        };

        retval.to_owned()
    }
}
