use crate::{get_makeopts_string, run_callback, Callbacks, LogType, SmbuilderError};
use n64romconvert::{determine_format, RomType};
use std::fs;
use std::path::Path;
use std::{fmt::Debug, io::Error};

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
/// Represents the region of a given ROM file.
pub enum Region {
    #[default]
    /// A rom pulled from a US cartridge.
    US,

    /// A rom pulled from a European cartridge (EU).
    EU,
    /// A rom pulled from a Japanese cartridge.
    JP,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Represents a ROM file.
pub struct Rom {
    /// The Region of the ROM Cartridge that
    /// the ROM was pulled from.
    pub region: Region,
    /// The path of the ROM file on disk.
    pub path: PathBuf,
    /// The format of the ROM file.
    pub format: RomType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Represents a git repository with the
/// source code of the a port.
pub struct Repo {
    /// The name of the repository.
    ///
    /// Used for launchers where
    /// the name may need to be a
    /// little bit more user friendly.
    pub name: String,
    /// The link to the repository.
    pub url: String,
    /// The branch to clone from.
    pub branch: String,
    /// The description of what the
    /// repo is, useful for launchers.
    pub about: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
/// Represents a key-value pair
/// Make Flag, such as `BETTERCAMERA=1`
pub struct Makeopt {
    /// The key of the flag.
    pub key: String,
    /// The value of the flag.
    pub value: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
/// Represents a data pack (DynOS).
pub struct Datapack {
    /// The label of the pack,
    /// for the launcher.
    pub label: String,
    /// Where the location of
    /// the pack is on disk.
    pub path: PathBuf,
    /// If the pack is enabled
    /// or not. Used for the
    /// hard-disable functionality.
    pub enabled: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
/// Represents a texture pack.
pub struct TexturePack {
    /// Where the location of
    /// the pack is on disk.
    pub path: PathBuf,
    /// If the pack is enabled
    /// or not. Used for the
    /// hard-disable feature.
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize)]
/// Represents a build spec.
///
/// All of its child structs implements
/// `Deserialize` and `Serialize`, and a
/// spec file is derived directly from this
/// structure.
pub struct Spec {
    /// The ROM to extract assets out of.
    pub rom: Rom,
    /// The repository to build from.
    pub repo: Repo,
    /// Amount of compile jobs that are
    /// allowed for the compiler. Will
    /// be used to set the `-j` flag
    /// during compile time.
    pub jobs: Option<u8>,
    /// A custom name.
    pub name: Option<String>,
    /// Make flags to be passed to the
    /// compiler.
    pub makeopts: Option<Vec<Makeopt>>,
    /// A texture pack, if supported.
    pub texture_pack: Option<TexturePack>,
    /// Datapacks, if supported.
    pub packs: Option<Vec<Datapack>>,
}

// TODO: write a SpecBuilder
impl Spec {
    /// # Please do not use this.
    ///
    /// **
    /// It's only for users of
    /// this crate that will perform
    /// checks themselves, or
    /// masochists!
    /// **
    ///
    /// Creates a new spec, from a file,
    /// but **doesn't check it**, which **may
    /// lead to random panics**
    ///
    /// # Example
    /// `Hey, you. why are you here? You shouldn't be using this at all!`
    pub fn from_file_unchecked<P: AsRef<Path>>(path: P) -> Result<Spec, SmbuilderError> {
        let file_string = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                return Err(SmbuilderError::new(
                    Some(Box::new(e)),
                    "Failed to read the file",
                ))
            }
        };

        let retval = match serde_yaml::from_str::<Spec>(&file_string) {
            Ok(s) => s,
            Err(e) => {
                return Err(SmbuilderError::new(
                    Some(Box::new(e)),
                    "Failed to parse the file into a yaml",
                ))
            }
        };

        Ok(retval)
    }

    pub fn check_spec(&mut self, callbacks: &mut Callbacks) -> Result<(), SmbuilderError> {
        use LogType::*;

        // Check the ROM format and see
        // if it matches the spec
        let rom_path = if self.rom.path.exists() {
            &self.rom.path
        } else {
            let file_not_found_error = std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("the file at {} was not found!", &self.rom.path.display()),
            );
            return Err(SmbuilderError::new(
                Some(Box::new(file_not_found_error)),
                "the ROM at the given path was not found!",
            ));
        };

        let verified_rom_format = match determine_format(rom_path) {
            Ok(t) => t,
            Err(e) => {
                return Err(SmbuilderError::new(
                    Some(Box::new(e)),
                    "failed to verify the ROM's format",
                ))
            }
        };

        if verified_rom_format != self.rom.format {
            run_callback!(
                callbacks.log_cb,
                Warn,
                &format!(
                    "the ROM format specified in the spec ({:?}) does not match the file ({:?})!",
                    self.rom.format, verified_rom_format
                )
            );
        };

        // Repo
        // TODO: finnish writing the repo metadata first

        // Jobs

        if self.jobs.is_none() {
            run_callback!(
                callbacks.log_cb,
                Warn,
                "did not find a value for jobs in the spec!"
            );

            run_callback!(
                callbacks.log_cb,
                Warn,
                "it is highly advised for you to specify the variable!"
            );
        }

        Ok(())
    }

    // TODO: write a from_file function that checks

    /// Gets a build shell script, ready to be
    /// written to disk.
    ///
    /// TODO: example

    // TODO: platform dependent code
    pub fn get_build_script(&self, repo_path: &Path) -> String {
        let makeopts_string = if let Some(makeopts) = &self.makeopts {
            get_makeopts_string(makeopts)
        } else {
            String::new()
        };

        let jobs = self.jobs.unwrap_or(2);

        format!(
            "
#!/bin/sh

# Script Generated by smbuilder.
# DO NOT EDIT; YOUR CHANGES
# WILL NOT BE SAVED.

make -C {} {} -j{}
        ",
            repo_path.display(),
            makeopts_string,
            jobs
        )
    }
}
