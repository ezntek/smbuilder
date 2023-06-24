use duct::cmd;
use git2::build::RepoBuilder;
use git2::{FetchOptions, RemoteCallbacks};
use n64romconvert::{byte_swap, endian_swap, RomType};

use crate::prelude::{run_callback, Callbacks, LogType, Region};
use crate::SmbuilderError;
use crate::{make_file_executable, prelude::Spec};
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use LogType::*;
use SetupStage::*;

#[derive(Debug)]
/// An enum to represent the different "setup stages"
/// involved in building a port.
///
/// It includes critical steps to be able to
/// build a basic, vanilla port.
pub enum SetupStage {
    /// Write the spec file to disk.
    WriteSpec,

    /// Clone the repository from
    /// the spec.
    CloneRepo,

    /// Copy the base ROM (and converts
    /// its format, if necessary) into
    /// the repo's root for asset extraction.
    CopyRom,

    /// Create the build script.
    CreateBuildScript,

    /// Create the post-build scripts directory,
    CreateScriptsDir,

    /// Write the Post-build scripts to disk.
    WritePostBuildScripts,
}

impl ToString for SetupStage {
    fn to_string(&self) -> String {
        use SetupStage::*;

        let result = match self {
            WriteSpec => "write the spec to disk",
            CloneRepo => "clone the repository",
            CopyRom => "copy the base ROM",
            CreateBuildScript => "create the build script",
            CreateScriptsDir => "create the post-build script folder",
            WritePostBuildScripts => "write the post-build scripts",
        };

        result.to_owned()
    }
}

/// The main builder class which takes care of building
/// a spec.
///
/// Includes fields and methods that makes up the core of
/// any smbuilder-derived app.
///
/// # Example
///
/// ```rust
/// use smbuilder::prelude::*;
///
/// fn main() {
///     // set your callbacks up first
///     let mut callbacks = Callbacks::empty();
///
///     // and your spec
///     let my_spec = Spec::from_file("path/to/my/smbuilder.yaml")
///
///     // set up your builder
///     let mut builder = Builder::new(my_spec, "path/to/the/root/dir", mut callbacks);
///
///     // compile the spec, with the specified callbacks.
///     builder.build();
/// }
///
/// ```
///
pub struct Builder<'a> {
    /// The spec that the build will be built with.
    pub spec: Spec,

    /// The base directory, the dir where the spec lives
    pub base_dir: PathBuf,

    /// The logger.
    pub callbacks: Callbacks<'a>,
}

impl<'a> Builder<'a> {
    /// Creates a new `Builder`.
    ///
    /// It creates the base directory from
    /// the parameter `root_dir`, which **is not**
    /// the root directory of your disk, rather
    /// it is the parent directory of where the
    /// base directory will be. Useful if one
    /// chooses to put it into a location such as
    /// `$HOME/.local/share`, for frontends.
    ///
    /// It takes in the callbacks, for events that
    /// may happen during the build process.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # fn main() {
    /// let mut builder = Builder::new(my_spec, my_root_dir, my_callbacks)
    /// // you must have your spec, root dir and callbacks set up beforehand!
    /// # }
    /// ```
    pub fn new<P: AsRef<Path>>(
        spec: Spec,
        root_dir: P,
        mut callbacks: Callbacks,
    ) -> Result<Builder, SmbuilderError> {
        let base_dir = Builder::create_base_dir(&spec, root_dir, &mut callbacks);

        let result = Builder {
            spec,
            base_dir,
            callbacks,
        };

        Ok(result)
    }

    fn create_base_dir<P: AsRef<Path>>(
        spec: &Spec,
        root_dir: P,
        callbacks: &mut Callbacks,
    ) -> PathBuf {
        // this function runs before `new`,
        // so this will not take in self, but
        // will return the result that is relevant.

        let base_dir_name = if let Some(name) = &spec.name {
            name
        } else {
            &spec.repo.name
        };

        run_callback!(&mut callbacks.log_cb, Info, "creating the base directory");

        let unconfirmed_base_dir = root_dir.as_ref().join(base_dir_name);
        let base_dir = if unconfirmed_base_dir.exists() {
            return unconfirmed_base_dir;
        } else {
            unconfirmed_base_dir
        };

        fs::create_dir(&base_dir)
            .unwrap_or_else(|_| panic!("failed to create a directory at {:?}", &base_dir));

        base_dir
    }

    /// Writes the current spec to disk.
    pub fn write_spec(&mut self) {
        run_callback!(self.callbacks.new_stage_cb, WriteSpec);

        let file_path = self.base_dir.join("smbuilder.yaml");

        run_callback!(
            self.callbacks.log_cb,
            Info,
            &format!("creating the spec file at {}", &file_path.display())
        );

        let mut smbuilder_specfile = fs::File::create(&file_path).unwrap_or_else(|_| {
            panic!(
                "failed to create the spec file at {}: ",
                &file_path.display()
            )
        });

        run_callback!(
            self.callbacks.log_cb,
            Info,
            &format!(
                "writing the contents of the spec into {}",
                &file_path.display()
            )
        );

        smbuilder_specfile
            .write_all(serde_yaml::to_string(&self.spec).unwrap().as_bytes())
            .unwrap_or_else(|_| {
                panic!(
                    "failed to write the spec into the file at {}: ",
                    &file_path.display()
                )
            });
    }

    /// Clones the repository as specified in the
    /// spec to disk.
    pub fn clone_repo(&mut self) -> PathBuf {
        run_callback!(self.callbacks.new_stage_cb, CloneRepo);

        let repo_name = &self.spec.repo.name;
        let repo_dir = Arc::new(self.base_dir.join(repo_name));

        run_callback!(self.callbacks.log_cb, Info, "cloning the repository");

        let repo_dir_thread = Arc::clone(&repo_dir);
        // set up the ctrlc handler
        ctrlc::set_handler(move || {
            let repo_dir = (*(repo_dir_thread.clone())).clone();
            println!("exiting on control-c...");

            if !repo_dir.exists() {
                std::process::exit(0);
            }

            fs::remove_dir_all(&repo_dir).unwrap_or_else(|e| {
                panic!("failed to remove the dir at {}: {}", &repo_dir.display(), e)
            });

            std::process::exit(0);
        })
        .expect("failed to set the control-c handler!");

        let mut remote_callbacks = RemoteCallbacks::new();
        remote_callbacks.transfer_progress(|progress| {
            run_callback!(
                self.callbacks.repo_clone_progress_cb,
                progress.received_objects(),
                progress.total_objects(),
                progress.received_bytes(),
            );

            true
        });

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(remote_callbacks);

        RepoBuilder::new()
            .branch(&self.spec.repo.branch)
            .fetch_options(fetch_options)
            .clone(&self.spec.repo.url, &*repo_dir)
            .unwrap_or_else(|_| {
                panic!(
                    "failed to clone the repository from {} into {}: ",
                    &self.spec.repo.url,
                    &repo_dir.display()
                )
            });

        (*repo_dir).clone()
    }

    /// Copies the ROM from the path specified
    /// in the spec into the root of the repo,
    /// performing a format conversion if
    /// necessary.
    pub fn copy_rom<P: AsRef<Path>>(&mut self, repo_dir: P) {
        run_callback!(self.callbacks.new_stage_cb, CopyRom);
        use RomType::*;

        let rom_type = self.spec.rom.format;
        let target_rom_path = repo_dir
            .as_ref()
            .join(format!("baserom.{}.z64", self.spec.rom.region.to_string()));

        run_callback!(self.callbacks.log_cb, Info, "copying the ROM");

        if rom_type == BigEndian {
            fs::copy(&self.spec.rom.path, &target_rom_path).unwrap_or_else(|_| {
                panic!(
                    "failed to copy the ROM from {} to {}!",
                    &self.spec.rom.path.display(),
                    target_rom_path.display()
                )
            });
        } else {
            run_callback!(
                self.callbacks.log_cb,
                Warn,
                "the ROM is not a z64 format ROM!"
            );
            run_callback!(
                self.callbacks.log_cb,
                Warn,
                &format!("converting from a {:?} ROM", rom_type)
            );

            match rom_type {
                LittleEndian => endian_swap(&self.spec.rom.path, &target_rom_path),
                ByteSwapped => byte_swap(&self.spec.rom.path, &target_rom_path),
                _ => unreachable!(),
            }
        }
    }

    /// Creates the build script in the `base_dir`.
    pub fn create_build_script<P: AsRef<Path>>(&mut self, repo_dir: P) {
        run_callback!(self.callbacks.new_stage_cb, CreateBuildScript);

        let file_path = self.base_dir.join("build.sh");

        run_callback!(self.callbacks.log_cb, Info, "arstarstarst");

        let mut build_script =
            fs::File::create(&file_path).expect("failed to create the build script file!");

        let build_script_contents = self.spec.get_build_script(repo_dir.as_ref());

        build_script
            .write_all(build_script_contents.as_bytes())
            .unwrap_or_else(|_| {
                panic!(
                    "failed to write to the build script at {}!",
                    &file_path.display()
                )
            });

        make_file_executable(&file_path)
    }

    /// Creates the post-build scripts
    /// directory at `repo_dir/scripts`.
    pub fn create_scripts_dir<P: AsRef<Path>>(&mut self, base_dir: P) -> PathBuf {
        run_callback!(self.callbacks.new_stage_cb, CreateScriptsDir);

        let scripts_dir = base_dir.as_ref().join("scripts");

        if !scripts_dir.exists() {
            fs::create_dir(&scripts_dir)
                .unwrap_or_else(|e| panic!("failed to create the build scripts dir: {}", e));
        }

        scripts_dir
    }

    /// Write the post-build scripts
    /// from the spec onto disk.
    pub fn write_scripts<P: AsRef<Path>>(&mut self, scripts_dir: P) {
        run_callback!(self.callbacks.new_stage_cb, WritePostBuildScripts);

        if let Some(scripts) = &mut self.spec.scripts {
            for script in scripts {
                let script_path = script.save(&scripts_dir);

                make_file_executable(&script_path);
            }
        }
    }

    fn setup_build(&mut self) {
        use SetupStage::*;

        let needed_targets =
            get_needed_setup_tasks(&self.spec, &self.base_dir, &mut self.callbacks);

        let repo_dir = self.base_dir.join(&self.spec.repo.name);
        let scripts_dir = repo_dir.join("scripts");

        for target in needed_targets {
            match target {
                WriteSpec => self.write_spec(),
                CloneRepo => {
                    let _ = self.clone_repo();
                }
                CopyRom => {
                    self.copy_rom(&repo_dir);
                }
                CreateBuildScript => {
                    self.create_build_script(&repo_dir);
                }
                CreateScriptsDir => {
                    let _ = self.create_scripts_dir(self.base_dir.clone());
                }
                WritePostBuildScripts => self.write_scripts(&scripts_dir),
            }
        }
    }

    fn compile(&mut self) {
        let build_cmd = cmd!(self.base_dir.join("build.sh")).stderr_to_stdout();
        let output = build_cmd
            .reader()
            .unwrap_or_else(|e| panic!("failed to get a reader from the command: {}", e));
        let reader = BufReader::new(output);

        for line in reader.lines() {
            let ln = match line {
                Ok(line) => line,
                Err(e) => panic!("The build command failed to run: {}", e),
            }; // exit when there is no more output

            run_callback!(self.callbacks.log_cb, BuildOutput, &ln);
        }
    }

    fn post_build(&mut self) {
        if let Some(scripts) = &self.spec.scripts {
            for script in scripts {
                run_callback!(
                    self.callbacks.new_postbuild_script_cb,
                    &script.name,
                    &script.description
                );

                let script_path = script.path.as_ref().unwrap_or_else(|| {
                    panic!("failed to unwrap the script path (please report this bug!)")
                });

                let cmd = cmd!(script_path);
                cmd.run()
                    .unwrap_or_else(|e| panic!("failed to run the command: {}", e));
            }
        }
    }

    /// Build the spec.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let mut builder = Builder::new(my_spec, my_root_dir, my_callbacks);
    /// // you must have your spec, root dir and callbacks set up beforehand!
    ///
    /// // builds the spec, takes a mutable reference
    /// // to itself for the callbacks.
    /// builder.build();
    /// ```
    pub fn build(&mut self) {
        self.setup_build();

        let executable_name = format!("sm64.{}.f3dex2e", self.spec.rom.region.to_string());

        let executable_path = self
            .base_dir
            .join(&self.spec.repo.name)
            .join("build")
            .join(format!("{}_pc", self.spec.rom.region.to_string()))
            .join(executable_name);

        if !executable_path.exists() {
            self.compile();
        } else {
            run_callback!(
                self.callbacks.log_cb,
                LogType::Warn,
                &format!(
                    "not building the spec: the executable at {} already exists!",
                    executable_path.display()
                )
            );
        }

        self.post_build();
    }
}

/// Get the core setup tasks that are needed.
///
/// Returns a list of `SmbuilderSetupStage`.
// TODO: example
pub fn get_needed_setup_tasks<P: AsRef<Path>>(
    spec: &Spec,
    base_dir: P,
    callbacks: &mut Callbacks,
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
        callbacks.log_cb,
        Info,
        &format!("needed tasks: {}", needed_stages_string)
    );

    // post-build script stuff
    if !base_dir.join("scripts").exists() {
        needed_stages.push(CreateScriptsDir)
    }

    if let Some(scripts) = &spec.scripts {
        for script in scripts {
            if script.path.is_none() {
                needed_stages.push(WritePostBuildScripts);
                continue;
            }
        }
    }

    // return
    needed_stages
}

impl ToString for Region {
    fn to_string(&self) -> String {
        use Region::*;

        let retval = match self {
            Us => "us",
            Eu => "eu",
            Jp => "jp",
            Sh => "sh",
        };

        retval.to_owned()
    }
}
