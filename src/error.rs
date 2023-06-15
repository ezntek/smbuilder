use std::{error::Error, fmt::Display};

use colored::Colorize;

use crate::prelude::{CmdoutSettings, Settings};
#[derive(Debug)]
pub struct SmbuilderError {
    cause: Option<Box<dyn Error>>,
    description: String,
}

impl SmbuilderError {
    pub fn new<S: AsRef<str>>(cause: Option<Box<dyn Error>>, description: S) -> Self {
        SmbuilderError {
            cause,
            description: description.as_ref().to_owned(),
        }
    }

    pub fn pretty_panic(self, settings: &Settings) {
        let runnable_settings = (*settings).get_runnable();
        runnable_settings.error(&self.description);

        let panic_text = if let CmdoutSettings::Silent = settings.cmdout_settings {
            &self.description
        } else {
            "panic from a pretty-panic. check error message above."
        };

        // goodbye, program ;)
        panic!("{}", panic_text)
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

impl Error for SmbuilderError {
    fn cause(&self) -> Option<&dyn Error> {
        if let Some(e) = &self.cause {
            Some(&**e)
        } else {
            None
        }
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
