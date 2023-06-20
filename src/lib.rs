#![doc = include_str!("../doc/crate.md")]
#![warn(missing_docs)]

pub mod builder;

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

pub enum LogType {
    Error,
    Warn,
    BuildOutput,
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
// eason@eznetek.com
// please
// thanks

pub type LogCallback<'cb> = dyn Fn(LogType, &str) + 'cb;
pub type NewStageCallback<'cb> = dyn Fn(&SetupStage) + 'cb;

pub struct BuilderCallbacks<'cb> {
    pub log_cb: Option<Box<LogCallback<'cb>>>,
    pub new_stage_cb: Option<Box<NewStageCallback<'cb>>>,
}

impl<'cb> BuilderCallbacks<'cb> {
    pub fn empty() -> Self {
        BuilderCallbacks {
            log_cb: None,
            new_stage_cb: None,
        }
    }

    pub fn log<F>(mut self, callback: F) -> Self
    where
        F: Fn(LogType, &str) + 'cb,
    {
        self.log_cb = Some(Box::new(callback) as Box<LogCallback<'cb>>);
        self
    }

    pub fn new_stage<F>(mut self, callback: F) -> Self
    where
        F: Fn(&SetupStage) + 'cb,
    {
        self.new_stage_cb = Some(Box::new(callback) as Box<NewStageCallback<'cb>>);
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
pub fn make_file_executable(path: &Path) -> Result<(), SmbuilderError> {
    let file_metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(e) => {
            return Err(SmbuilderError::new(
                Some(Box::new(e)),
                format!(
                    "failed to get the metadata of the file at {}",
                    &path.display()
                ),
            ))
        }
    };

    match fs::set_permissions(
        path,
        fs::Permissions::from_mode(
            file_metadata.permissions().mode() + 0o111, // equivalent of a chmod +x.
        ),
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(SmbuilderError::new(
            Some(Box::new(e)),
            format!(
                "failed to set permissions on the file at {}",
                &path.display(),
            ),
        )),
    }
}
