use std::fmt::Display;

use colored::Colorize;

#[derive(Debug)]
pub struct Error {
    pub cause: Option<Box<dyn std::error::Error>>,
    pub description: String,
}

impl Error {
    pub fn new<S: AsRef<str>>(cause: Option<Box<dyn std::error::Error>>, description: S) -> Self {
        Error {
            cause,
            description: description.as_ref().to_owned(),
        }
    }

    /*pub fn pretty_panic(self, settings: &Settings) {
        let runnable_settings = (*settings).to_runnable();

        let panic_text = if let Some(e) = &self.cause {
            format!("{}: {}", self.description, *e)
        } else {
            format!("{}", self.description)
        };

        runnable_settings.error(&panic_text);

        // goodbye, program ;)
        if let CmdoutSettings::Silent = settings.cmdout_settings {
            panic!("{}", panic_text)
        } else {
            panic!("panicked from a pretty-panic: check the error message above.")
        }
    }*/
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let displayed_string = if let Some(e) = &self.cause {
            format!("{}{}: {}", "error: ".bold().red(), self.description, *e)
        } else {
            format!("{}{}", "error: ".bold().red(), self.description,)
        };

        write!(f, "{}", displayed_string)
    }
}

impl std::error::Error for Error {
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
