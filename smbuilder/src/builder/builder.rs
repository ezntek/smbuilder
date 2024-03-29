use super::get_needed_setup_tasks;
use super::types::BuilderResult;
use super::types::{
    PostBuildStage::*,
    SetupStage::{self, *},
};

use crate::callback_types::LogType::{self, *};
use crate::callbacks::run_callback;
use crate::error::ErrorCause;
use crate::prelude::error_macros::*;
use crate::prelude::{err, Callbacks, Error, Spec};
use crate::util;

use duct::cmd;
use git2::build::RepoBuilder;
use git2::{FetchOptions, RemoteCallbacks};
use n64romconvert::{byte_swap, endian_swap, RomType};
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

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
/// // set your callbacks up first
/// let mut callbacks = Callbacks::empty();
///
/// // and your spec
/// let my_spec = Spec::from_file("path/to/my/smbuilder.yaml")
///
/// // set up your builder
/// let mut builder = Builder::new(my_spec, "path/to/the/base/dir", mut callbacks);
///
/// // compile the spec, with the specified callbacks.
/// builder.build();
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
    /// It takes in the callbacks, for events that
    /// may happen during the build process.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # fn main() {
    /// let mut builder = Builder::new(my_spec, my_base_dir, my_callbacks)
    /// // you must have your spec, base dir and callbacks set up beforehand!
    /// # }
    /// ```
    pub fn new<P: Into<PathBuf>>(
        spec: Spec,
        base_dir: P,
        callbacks: Callbacks,
    ) -> Result<Builder, Error> {
        let result = Builder {
            spec,
            base_dir: base_dir.into(),
            callbacks,
        };

        Ok(result)
    }

    fn clone_repo(&mut self) -> BuilderResult<PathBuf> {
        run_callback!(self.callbacks.new_setup_stage_cb, CloneRepo);

        let repo_name = &self.spec.repo.name;
        let repo_dir = Arc::new(self.base_dir.join(repo_name));

        run_callback!(self.callbacks.log_cb, Info, "cloning the repository");

        // set up the ctrlc handler
        let repo_dir_thread = Arc::clone(&repo_dir);
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
        fetch_options
            .remote_callbacks(remote_callbacks)
            .follow_redirects(git2::RemoteRedirect::All);

        let repo_clone_result = RepoBuilder::new()
            .branch(&self.spec.repo.branch)
            .fetch_options(fetch_options)
            .clone(&self.spec.repo.url, &repo_dir);

        match repo_clone_result {
            Ok(_) => (),
            Err(e) => {
                let msg = e.message().to_string();
                let err = err!(
                    c_repo_clone!(self.spec.repo.url.clone(), (*repo_dir).clone(), e),
                    format!("failed to clone the repository: {}", msg)
                );
                return Err(err);
            }
        }

        Ok((*repo_dir).clone())
    }

    fn copy_rom<P: AsRef<Path>>(&mut self, repo_dir: P) -> BuilderResult<()> {
        run_callback!(self.callbacks.new_setup_stage_cb, CopyRom);
        use RomType::*;

        let rom_type = self.spec.rom.format;
        let target_rom_path = repo_dir
            .as_ref()
            .join(format!("baserom.{}.z64", self.spec.rom.region.to_string()));

        run_callback!(self.callbacks.log_cb, Info, "copying the ROM");

        if rom_type == BigEndian {
            match fs::copy(&self.spec.rom.path, &target_rom_path) {
                Ok(_) => Ok(()),
                Err(e) => {
                    let msg = format!(
                        "failed to copy the ROM from {} to {}!",
                        &self.spec.rom.path.display(),
                        target_rom_path.display()
                    );
                    return Err(err!(c_fs!(e, msg), "whilst copying the ROM file"));
                }
            }
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
            };

            Ok(())
        }
    }

    fn create_build_script<P: AsRef<Path>>(&mut self, repo_dir: P) -> BuilderResult<()> {
        run_callback!(self.callbacks.new_setup_stage_cb, CreateBuildScript);

        let file_path = self.base_dir.join("build.sh");

        let mut build_script =
            fs::File::create(&file_path).expect("failed to create the build script file!");

        let build_script_contents = self.spec.to_script(repo_dir.as_ref());

        match build_script.write_all(build_script_contents.as_bytes()) {
            Ok(_) => (),
            Err(e) => {
                let msg = format!(
                    "failed to write to the build script at {}!",
                    &file_path.display()
                );
                return Err(err!(c_fs!(e, msg), "whilst writing the build script"));
            }
        };

        util::make_file_executable(&file_path);
        Ok(())
    }

    fn create_scripts_dir<P: AsRef<Path>>(&mut self, base_dir: P) -> BuilderResult<PathBuf> {
        run_callback!(self.callbacks.new_setup_stage_cb, CreateScriptsDir);

        let scripts_dir = base_dir.as_ref().join("scripts");

        if !scripts_dir.exists() {
            match fs::create_dir(&scripts_dir) {
                Ok(_) => (),
                Err(e) => {
                    let msg = format!("failed to create the build scripts dir: {}", e);
                    return Err(err!(
                        c_fs!(e, msg),
                        "whilst trying to create the build script directory"
                    ));
                }
            }
        }

        Ok(scripts_dir)
    }

    fn write_scripts<P: AsRef<Path>>(&mut self, scripts_dir: P) {
        run_callback!(self.callbacks.new_setup_stage_cb, WritePostBuildScripts);

        if let Some(scripts) = &mut self.spec.scripts {
            for script in scripts {
                let script_path = script.save(&scripts_dir).unwrap(); // BUG: unwrap

                util::make_file_executable(&script_path);
            }
        }
    }

    fn setup_build(&mut self) -> BuilderResult<()> {
        use SetupStage::*;

        let needed_targets =
            get_needed_setup_tasks(&self.spec, &self.base_dir, &mut self.callbacks);

        let repo_dir = self.base_dir.join(&self.spec.repo.name);
        let scripts_dir = repo_dir.join("scripts");

        for target in needed_targets {
            match target {
                CloneRepo => {
                    let _ = self.clone_repo();
                }
                CopyRom => {
                    self.copy_rom(&repo_dir)?;
                }
                CreateBuildScript => {
                    self.create_build_script(&repo_dir)?;
                }
                CreateScriptsDir => {
                    let _ = self.create_scripts_dir(self.base_dir.clone())?;
                }
                WritePostBuildScripts => self.write_scripts(&scripts_dir),
            }
        }

        Ok(())
    }

    fn compile(&mut self) {
        let build_script_path = self.base_dir.join("build.sh").canonicalize().unwrap();
        dbg!(&build_script_path);
        let build_cmd = cmd!(build_script_path).stderr_to_stdout();
        let output = build_cmd
            .reader()
            .unwrap_or_else(|e| panic!("failed to get a reader from the command: {}", e));
        let reader = BufReader::new(output);

        for line in reader.lines() {
            let ln = match line {
                Ok(line) => line,
                Err(e) => panic!("something went wrong: {}", e),
            }; // exit when there is no more output

            run_callback!(self.callbacks.log_cb, BuildOutput, &ln);
        }
    }

    fn install_texture_pack(&mut self) -> BuilderResult<()> {
        run_callback!(self.callbacks.new_postbuild_stage_cb, TexturePack);

        let pack = if let Some(pack) = &self.spec.texture_pack {
            pack
        } else {
            return Ok(());
        };

        let repo_dir = &self.base_dir.join(&self.spec.repo.name);

        pack.install(&self.spec, repo_dir)?;

        Ok(())
    }

    fn install_dynos_packs(&mut self) -> BuilderResult<()> {
        run_callback!(self.callbacks.new_postbuild_stage_cb, DynOSPacks);

        let packs = if let Some(packs) = &self.spec.dynos_packs {
            packs
        } else {
            return Ok(());
        };

        let repo_dir = &self.base_dir.join(&self.spec.repo.name);

        for pack in packs {
            pack.install(&self.spec, repo_dir, &mut self.callbacks)?;
        }

        Ok(())
    }

    fn run_postbuild_scripts(&mut self) -> BuilderResult<()> {
        run_callback!(self.callbacks.new_postbuild_stage_cb, PostBuildScripts);

        let scripts = if let Some(scripts) = &self.spec.scripts {
            scripts
        } else {
            return Ok(());
        };

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
            match cmd.run() {
                Ok(_) => (),
                Err(e) => {
                    return Err(err!(
                        c_spawn_cmd!(
                            script_path.to_string_lossy().to_string(),
                            "failed to run the script",
                            e
                        ),
                        format!("whilst trying to run script {}", script.name)
                    ))
                }
            };
        }

        Ok(())
    }

    fn post_build(&mut self) -> BuilderResult<()> {
        self.install_texture_pack()?;
        self.install_dynos_packs()?;
        self.run_postbuild_scripts()?;

        Ok(())
    }

    /// Build the spec.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let mut builder = Builder::new(my_spec, my_base_dir, my_callbacks);
    /// // you must have your spec, base dir and callbacks set up beforehand!
    ///
    /// // builds the spec, takes a mutable reference
    /// // to itself for the callbacks.
    /// builder.build();
    /// ```
    pub fn build(&mut self) -> BuilderResult<()> {
        self.setup_build()?;

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

        self.post_build()?;
        Ok(())
    }
}
