use crate::prelude::SmbuilderWrapper;
use crate::SmbuilderError;

pub trait Smbuilder {
    fn setup_build(&self, wrapper: &SmbuilderWrapper);
    fn build(&self, wrapper: &SmbuilderWrapper) -> Result<(), SmbuilderError>;
}
