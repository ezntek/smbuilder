#![warn(missing_docs)]

//! # smbuilder -- API documentation
//!
//! smbuilder is a small rust crate that provides an interface for the compilation of various ports within the family of ports of Super Mario 64 to the PC.
//!
//! The crate:
//!  * Provides strong types and models that wraps around the resources needed and other "moving parts" (texture packs, etc) for a given port
//!  * Allows for the types to be serialized/deserialized into/from yaml for reproducible build specifications (specs)
//!  * Provides classes and traits to provide an interface to build specs.
//!
//! ## Notes
//!
//! * This crate is not quite production quality yet. breaking API changes may come sooner or later.
//! * The bindings of these resources/moving parts for a build cannot be fully complete without actually modifying the port. However, more bindings may be added later.
//! * The choice of repositories and the makeopts supported by those ports should be handled by the app that uses this crate. However, makeopts may be implemented as enums/structs in a later version or in another crate.
//!
//! ## Usage
//!
//! WIP

/// The API and rust representation(s)
/// of core build processes that are
/// involved in building a port.
pub mod builder;

/// Enums related to some
/// common make flags that people
/// generally set.
pub mod makeopts;

/// All the logic and code
/// that relates to the
/// reproducible spec.
pub mod spec;

/// The prelude of this crate.
pub mod prelude;

/// Types that relate to the
/// settings of a build.

/// Core types that binds common
/// build resources to rust types.
pub mod types;

use colored::Colorize;
use prelude::*;
use std::fmt::Display;
use std::{fs, os::unix::prelude::PermissionsExt, path::Path};

#[macro_export]
/// A macro to make writing
/// a makeopt less painful.
///
/// TODO: example
macro_rules! makeopt {
    ($key:expr, $value:expr) => {
        Makeopt::new($key, $value)
    };
}

#[macro_export]
/// Run a `Callbacks`
/// callback, if it exists.
///
/// Does not panic if it
/// does not exist. You
/// also do need a mutable
/// reference to the callback,
/// as there are `FnMut`s.
///
/// TODO: example
macro_rules! run_callback {
    ($callback:expr, $($cb_arg:tt)*) => {
        if let Some(callback) = &mut $callback {
            callback($($cb_arg)*);
        };
    };
}

/// An enum to represent
/// a log type, for the
/// log callback.
pub enum LogType {
    /// Indicates an error.
    Error,
    /// Indicates a warning.
    Warn,
    /// Indicates some
    /// build output.
    BuildOutput,
    /// Indicates some
    /// info.
    Info,
}

#[derive(Debug)]
/// An smbuilder-related error.
pub struct SmbuilderError {
    /// The cause of the error.
    pub cause: Option<Box<dyn std::error::Error>>,

    /// The description of the error.
    pub description: String,
}

impl SmbuilderError {
    /// Creates a new `SmbuilderError`.
    pub fn new<S: AsRef<str>>(cause: Option<Box<dyn std::error::Error>>, description: S) -> Self {
        SmbuilderError {
            cause,
            description: description.as_ref().to_owned(),
        }
    }
}

impl Display for SmbuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let displayed_string = if let Some(e) = &self.cause {
            format!("{}{}: {}", "error: ".bold().red(), self.description, *e)
        } else {
            format!("{}{}", "error: ".bold().red(), self.description,)
        };

        write!(f, "{}", displayed_string)
    }
}

impl std::error::Error for SmbuilderError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        if let Some(e) = &self.cause {
            Some(&**e)
        } else {
            None
        }
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

// sexy callback time
// O.O
// im going insane help me
// eason@ezntek.com
// please
// thanks

/// Callback for a log event.
///
/// Args:
///  * log type (error, warning, info, etc.)
///  * text to log
pub type LogCallback<'cb> = dyn FnMut(LogType, &str) + 'cb;

/// Callback for a new setup stage.
///
/// Args:
///  * setup stage
pub type NewSetupStageCallback<'cb> = dyn FnMut(SetupStage) + 'cb;

/// Callback for when a new Post-Build script is run.
///
/// Args:
///  * filename of the script
///  * description of the script
pub type NewPostBuildScriptCallback<'cb> = dyn FnMut(&str, &str) + 'cb;

/// Callback for repository clone progress.
///
/// Args:
///  * the percentage (0.0 - 1.0)
///  * recieved bytes
pub type RepoCloneProgressCallback<'cb> = dyn FnMut(f64, usize) + 'cb;

/// A struct to store callbacks
/// to hook various events during
/// the build lifecycle to various
/// functions.
///
/// TODO: example
pub struct Callbacks<'cb> {
    /// The log callback.
    pub log_cb: Option<Box<LogCallback<'cb>>>,
    /// A callback that is invoked
    /// on a new setup stage.
    pub new_stage_cb: Option<Box<NewSetupStageCallback<'cb>>>,
    /// A callback that is invoked when
    /// a new post-build script is being
    /// run.
    pub new_postbuild_script_cb: Option<Box<NewPostBuildScriptCallback<'cb>>>,
    /// A callback that pipes information
    /// from git2's `RemoteCallbacks` to
    /// provide info on clone progress.
    pub repo_clone_progress_cb: Option<Box<RepoCloneProgressCallback<'cb>>>,
}

impl<'cb> Callbacks<'cb> {
    /// Create an empty callbacks set.
    pub fn empty() -> Self {
        Callbacks {
            log_cb: None,
            new_stage_cb: None,
            new_postbuild_script_cb: None,
            repo_clone_progress_cb: None,
        }
    }

    /// Set the log callback.
    ///
    /// See the docs on `LogCallback`
    /// for more information on arguments.
    ///
    /// TODO: example
    pub fn log<F>(mut self, callback: F) -> Self
    where
        F: FnMut(LogType, &str) + 'cb,
    {
        self.log_cb = Some(Box::new(callback) as Box<LogCallback<'cb>>);
        self
    }

    /// Set the new setup stage
    /// callback.
    ///
    /// See the docs on `NewSetupStageCallback`
    /// for more information on arguments.
    ///
    /// TODO: example.
    pub fn new_stage<F>(mut self, callback: F) -> Self
    where
        F: FnMut(SetupStage) + 'cb,
    {
        self.new_stage_cb = Some(Box::new(callback) as Box<NewSetupStageCallback<'cb>>);
        self
    }

    /// Set the repo clone progress
    /// callback.
    ///
    /// See the docs on `RepoCloneProgressCallback`
    /// for more information on arguments.
    ///
    /// TODO: example
    pub fn repo_clone_progress<F>(mut self, callback: F) -> Self
    where
        F: FnMut(f64, usize) + 'cb,
    {
        self.repo_clone_progress_cb =
            Some(Box::new(callback) as Box<RepoCloneProgressCallback<'cb>>);
        self
    }

    /// Set the new post-build script
    /// callback.
    ///
    /// See the docs on `NewPostBuildScriptCallback`
    /// for more information on arguments.
    ///
    /// TODO: example
    pub fn new_postbuild_script<F>(mut self, callback: F) -> Self
    where
        F: FnMut(&str, &str) + 'cb,
    {
        self.new_postbuild_script_cb =
            Some(Box::new(callback) as Box<NewPostBuildScriptCallback<'cb>>);
        self
    }
}

/// Get a string of options in the format of
/// bourne shell variables from a list of `makeopt`,
/// for use with the `make` command.
pub fn get_makeopts_string(makeopts: &[Makeopt]) -> String {
    let mut result = makeopts
        .iter()
        .map(|makeopt| format!("{}={}", makeopt.key, makeopt.value))
        .collect::<Vec<String>>()
        .join(" ");

    result.push(' '); // pad the last character out, just in case
    result
}

/// Make a file executable.
/// Equivalent to `chmod +x`.
pub fn make_file_executable(path: &Path) {
    let file_metadata = fs::metadata(path).unwrap_or_else(|e| {
        panic!(
            "failed to get the metadata of the file at {}: {}",
            &path.display(),
            e
        )
    });

    fs::set_permissions(
        path,
        fs::Permissions::from_mode(
            file_metadata.permissions().mode() + 0o111, // equivalent of a chmod +x.
        ),
    )
    .unwrap_or_else(|e| {
        panic!(
            "failed to set permissions on the file at {}: {}",
            &path.display(),
            e
        )
    });
}
