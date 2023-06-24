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
    Us,
    /// A rom pulled from a European cartridge (EU).
    Eu,
    /// A rom pulled from a Japanese cartridge.
    Jp,
    /// A rom pulled from a Japanese Shindou cartridge.
    Sh,
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

impl Default for Rom {
    fn default() -> Self {
        Rom {
            region: Region::Us,
            path: PathBuf::new(),
            format: RomType::BigEndian,
        }
    }
}

impl Rom {
    /// Creates a new `Rom`.
    ///
    // TODO: example
    pub fn new<P: AsRef<Path>>(region: Region, path: P, rom_format: RomType) -> Self {
        Rom {
            region,
            path: path.as_ref().to_owned(),
            format: rom_format,
        }
    }
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
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
/// A structure to represent
/// a DynOS (Dynamic Options
/// System) Datapack.
///
/// These packs can store
/// either just options
/// (sets of settings/options),
/// or other metadata with it
/// (how model packs work).
///
// TODO: example
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
    /// The path to the build
    /// script on disk. Will be
    /// `None` if the script
    /// has not been written to
    /// disk yet.
    pub path: Option<PathBuf>,
}

impl Makeopt {
    /// Creates a new `Makeopt`.
    ///
    // TODO: example
    pub fn new<S: ToString>(key: S, value: S) -> Self {
        Makeopt {
            key: key.to_string(),
            value: value.to_string(),
        }
    }

    /// Gets a list of default
    /// makeopts with sane
    /// defaults, and options
    /// for the current OS.
    ///
    // TODO: example
    pub fn default_makeopts() -> Vec<Self> {
        let mut makeopts: Vec<Makeopt> = Vec::new();

        // make a macro to make life easier
        macro_rules! push_makeopt {
            ($key:expr,$value:expr) => {
                makeopts.push(Makeopt::new($key, $value))
            };
        }

        // enable external data
        push_makeopt!("EXTERNAL_DATA", "1");

        // force the modern APIs
        push_makeopt!("RENDER_API", "GL");
        push_makeopt!("WINDOW_API", "SDL2");
        push_makeopt!("AUDIO_API", "SDL2");
        push_makeopt!("CONTROLLER_API", "SDL2");

        // macOS stuff
        #[cfg(target_os = "macos")]
        #[cfg(target_arch = "x86_64")]
        {
            push_makeopt!("OSX_BUILD", "1");
            push_makeopt!("TARGET_ARCH", "x86_64-apple-darwin");
            push_makeopt!("TARGET_BITS", "64");
        };

        #[cfg(target_os = "macos")]
        #[cfg(target_arch = "aarch64")]
        {
            push_makeopt!("OSX_BUILD", "1");
            push_makeopt!("TARGET_ARCH", "aarch64-apple-darwin");
            push_makeopt!("TARGET_BITS", "64");
        };

        makeopts
    }
}

impl PostBuildScript {
    /// Creates a post-build script from
    /// a `Path`.
    ///
    // TODO: example
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
            path: None,
        }
    }

    /// Saves a post-build script from
    /// a `String` in memory to a
    /// File path.
    ///
    // TODO: example
    pub fn save<P: AsRef<Path>>(&mut self, scripts_dir: P) -> PathBuf {
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

        self.path = Some(script_path.clone());
        script_path
    }
}
