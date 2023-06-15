use crate::prelude::SmbuilderWrapper;

pub trait Smbuilder {
    fn setup_build(&self, wrapper: &SmbuilderWrapper);
    fn build(&self, wrapper: &SmbuilderWrapper) -> Result<(), crate::error::Error>;
}
