//! The API and rust representation(s) of core build processes that are involved in building a port.

use n64romconvert::{determine_format, RomType};

use crate::prelude::{BuilderCallbacks, LogType, Region};
use crate::SmbuilderError;
use crate::{make_file_executable, prelude::Spec};
use std::io::BufWriter;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use LogType::*;

macro_rules! run_callback {
    ($callback:expr, $($cb_arg:tt)*) => {
        if let Some(callback) = $callback {
            callback($($cb_arg)*);
        };
    };
}

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
/// ```rust
/// use smbuilder::prelude::*;
/// # struct MyLogger;
/// # impl Log for MyLogger {
/// #     fn log_error(&self, text: &str) {
/// #         println!("{}", text)
/// #     }
/// #     fn log_build_output(&self, text: &str) {
/// #         println!("{}", text)
/// #     }
/// #     fn log_warn(&self, text: &str) {
/// #         println!("{}", text)
/// #     }
/// #     fn log_info(&self, text: &str) {
/// #         println!("{}", text)
/// #     }
/// # }
/// struct MyBuilder;
///
/// impl Smbuilder for MyBuilder {
///     fn setup_build(&self, wrapper: &BuildWrapper) {
///         // Get the needed setup tasks first: it eliminates
///         // the need to unnecessarily perform all setup
///         // steps.
///         let needed_setup_tasks = get_needed_setup_tasks(
///             &wrapper.spec,
///             &wrapper.base_dir,
///             &MyLogger // see the docs for how to implement a logger
///         );
///
///         // run the setup tasks and handle
///         // errors as you see fit
///         wrapper.write_spec()
///             .unwrap();
///
///         wrapper.clone_repo()
///             .expect("failed to clone the repo!");
///
///         // log something with the logger, as you see fit.
///         (*wrapper.logger) // TODO: fix whatever this is
///             .log_info("done cloning the repo!");
///
///         // ...
///         // Contile handling the errors from those tasks.
///
///     }
///
///     fn build(&self, wrapper: &BuildWrapper) -> Result<(), SmbuilderError> {
///         // use a `std::process:Command` to run the
///         // build script at `base_dir/build.sh`
///         let mut command = std::process::Command::new(wrapper.base_dir.join("build.sh"));
///         
///         // spawn the command
///         let mut child = command.spawn().unwrap();
///         
///         // this blocks the current function!
///         // use a `Stdio::piped()` and a `BufReader`
///         // if you want to stream the build output
///         // into something.
///         match child.wait() {
///             Ok(_) => Ok(()),
///             Err(e) => Err(SmbuilderError::new(
///                Some(Box::new(e)),
///                "some error happened!"
///             )),
///         }
///     }
/// }
/// ```
///
/// implementors must do this as the different
/// use cases for this crate will need different
/// ways of handling errors that are thrown
/// from core functions like these. As an example,
/// a GUI may need to spawn an error dialog, but a
/// CLI may just panic, etc.
///
/// ## warning
///
/// It is highly recommended that you follow
/// the output from `get_needed_setep_tasks()`, as
/// it tells you which steps you would need to call.
///
/// If not, it is still important to perform all
/// tasks listed, as these tasks are required for a
/// build to function.
pub trait Smbuilder {
    /// The build setup function.
    ///
    /// This is where one implements custom logic
    /// to handle `SmbuilderError`s that may be
    /// returned from the wrapper's given setup functions.
    fn setup_build(&self, wrapper: &BuildWrapper);

    /// The build function.
    ///
    /// This is where one implements custom logic
    /// behind running the build script provided by
    /// `create_build_script` from the wrapper class.
    ///
    /// Should return an `SmbuilderError` if the build
    /// fails due to any reason (child process with a
    /// failure exit code, some other error, etc.)
    fn build(&self, wrapper: &BuildWrapper) -> Result<(), SmbuilderError>;
}

#[derive(Debug)]
/// An enum to represent the different "setup stages"
/// involved in building a port.
///
/// It includes critical steps to be able to
/// build a basic, vanilla port.
pub enum SetupStage {
    /// Write the spec file to disk.
    WriteSpec,

    /// Clone the repository
    /// as per the info in the spec.
    CloneRepo,

    /// Convert the ROM to a
    /// big-endian ROM.
    ///
    /// Needed for all ports to
    /// extract assets.
    ConvertRom,

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

impl ToString for SetupStage {
    fn to_string(&self) -> String {
        use SetupStage::*;

        let result = match self {
            WriteSpec => "write the spec to disk",
            CloneRepo => "clone the repository",
            ConvertRom => "convert the ROM to the correct format",
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
///
/// TODO: example
///
pub struct BuildWrapper<'a> {
    /// The spec that the build will be built with.
    pub spec: Spec,

    /// The base directory, the dir where the spec lives
    pub base_dir: PathBuf,

    /// The logger.
    pub callbacks: BuilderCallbacks<'a>,
    builder: Box<dyn Smbuilder>,
}

impl<'a> BuildWrapper<'a> {
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
    ///
    ///
    pub fn new<P: AsRef<Path>>(
        spec: Spec,
        root_dir: P,
        callbacks: BuilderCallbacks,
        builder: Box<dyn Smbuilder>,
    ) -> Result<BuildWrapper, SmbuilderError> {
        let base_dir = BuildWrapper::create_base_dir(&spec, root_dir, &callbacks)?;

        let result = BuildWrapper {
            spec,
            base_dir,
            callbacks,
            builder,
        };

        Ok(result)
    }

    fn create_base_dir<P: AsRef<Path>>(
        spec: &Spec,
        root_dir: P,
        callbacks: &BuilderCallbacks,
    ) -> Result<PathBuf, SmbuilderError> {
        // this function runs before `new`,
        // so this will not take in self, but
        // will return the result that is relevant.

        let base_dir_name = if let Some(name) = &spec.name {
            name
        } else {
            &spec.repo.name
        };

        run_callback!(&callbacks.log_cb, Info, "creating the base directory");

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

        run_callback!(
            &self.callbacks.log_cb,
            Info,
            &format!("creating the spec file at {}", &file_path.display())
        );

        let mut smbuilder_specfile = match fs::File::create(&file_path) {
            Ok(f) => BufWriter::new(f),
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

        run_callback!(
            &self.callbacks.log_cb,
            Info,
            &format!(
                "writing the contents of the spec into {}",
                &file_path.display()
            )
        );

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

        run_callback!(&self.callbacks.log_cb, Info, "cloning the repository");

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

        run_callback!(
            &self.callbacks.log_cb,
            Info,
            "copying the baserom into the correct location"
        );

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
    callbacks: &BuilderCallbacks,
) -> Vec<SetupStage> {
    use SetupStage::*;

    let base_dir = base_dir.as_ref();
    let mut needed_stages: Vec<SetupStage> = Vec::new();

    // check if the spec exists in the dir
    if !base_dir.join("smbuilder.yaml").exists() {
        needed_stages.push(WriteSpec)
    }

    // check if the repo is cloned
    if !base_dir.join(&spec.repo.name).exists() {
        needed_stages.push(CloneRepo)
    }

    // check if the ROM is in the correct format
    let format =
        determine_format(&spec.rom.path).expect("The ROM's format could not be recognized");

    if format != RomType::BigEndian {
        needed_stages.push(ConvertRom)
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

    run_callback!(
        &callbacks.log_cb,
        Info,
        &format!("needed tasks: {}", needed_stages_string)
    );

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
