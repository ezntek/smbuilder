use crate::prelude::*;
use std::{
    fmt::Debug,
    fs,
    io::{BufWriter, Write},
    path::Path,
};

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
/// Represents a patch.
pub struct Patch {
    /// The name (label) of
    /// the patch, for use
    /// with launchers,
    pub name: String,
    /// The location of the
    /// path file on disk.
    pub path: PathBuf,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DynosPack {
    /// The name of the
    /// DynOS pack, for
    /// use with launchers.
    pub name: String,

    /// The location of
    /// the pack, on disk.
    pub path: PathBuf,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
/// Represents a texture pack.
pub struct PostBuildScript {
    /// The name of the script,
    /// to be used as the file
    /// name, with a .sh appended.
    pub name: String,
    /// A human readable
    /// description of the
    /// script.
    pub description: String,
    /// The contents of the
    /// script, in shell format.
    pub contents: String,
}

impl Makeopt {
    pub fn new<S: ToString>(key: S, value: S) -> Self {
        Makeopt {
            key: key.to_string(),
            value: value.to_string(),
        }
    }

    pub fn default_platform_makeopts() -> Vec<Self> {
        let mut makeopts: Vec<Makeopt> = Vec::new();

        // macOS stuff
        #[cfg(target_os = "macos")]
        #[cfg(target_arch = "x86_64")]
        {
            makeopts.push(Makeopt::new("OSX_BUILD", "1"));
            makeopts.push(Makeopt::new("TARGET_ARCH", "x86_64-apple-darwin"));
            makeopts.push(Makeopt::new("TARGET_BITS", "64"));
        };

        #[cfg(target_os = "macos")]
        #[cfg(target_arch = "aarch64")]
        {
            makeopts.push(Makeopt::new("OSX_BUILD", "1"));
            makeopts.push(Makeopt::new("TARGET_ARCH", "aarch64-apple-darwin"));
            makeopts.push(Makeopt::new("TARGET_BITS", "64"));
        };

        makeopts
    }
}

impl PostBuildScript {
    pub fn from_file<S, P>(name: S, description: S, file: P) -> Self
    where
        S: ToString,
        P: AsRef<Path>,
    {
        let file_contents = fs::read_to_string(file)
            .unwrap_or_else(|e| panic!("failed to read the post build script: {}", e));

        PostBuildScript {
            name: name.to_string(),
            description: description.to_string(),
            contents: file_contents,
        }
    }

    pub fn save<P: AsRef<Path>>(&self, scripts_dir: P) -> PathBuf {
        let mut script_path = scripts_dir.as_ref().join(&self.name);
        script_path.set_extension("sh");

        let mut script_file = BufWriter::new(fs::File::create(&script_path).unwrap_or_else(|e| {
            panic!(
                "failed to create the file at {}: {}",
                script_path.display(),
                e
            )
        }));

        script_file
            .write_all(self.contents.as_bytes())
            .unwrap_or_else(|e| {
                panic!(
                    "failed to write the file to {}: {}",
                    script_path.display(),
                    e
                )
            });

        script_path
    }
}
