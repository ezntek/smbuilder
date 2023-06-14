use std::{error::Error, fmt::Display};

use colored::Colorize;
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
}

impl Display for SmbuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", "error: ".bold().red(), self.description)
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
