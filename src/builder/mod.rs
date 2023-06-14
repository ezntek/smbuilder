pub mod build;
pub mod types;

pub use build::*;
pub use types::*;

use crate::prelude::{Region, Spec};
use std::path::Path;

use self::types::SmbuilderSetupStage;

#[cfg(test)]
mod tests {
    use crate::error::SmbuilderError;

    #[test]
    fn test_smbuilder_error() {
        let e: Result<(), SmbuilderError> =
            Result::Err(SmbuilderError::new(None, "haha test error"));
        println!("{}", e.unwrap_err());
    }
}

pub fn get_needed_setup_tasks<P: AsRef<Path>>(
    spec: &Spec,
    base_dir: P,
) -> Vec<SmbuilderSetupStage> {
    use SmbuilderSetupStage::*;

    let base_dir = base_dir.as_ref();
    let mut needed_stages: Vec<SmbuilderSetupStage> = Vec::new();

    // check if the spec exists in the dir
    if !base_dir.join("smbuilder.yaml").exists() {
        needed_stages.push(WriteSpec)
    }

    // check if the rom exists
    if !base_dir
        .join(format!("baserom.{}.z64", spec.rom.region.to_string()))
        .exists()
    {
        needed_stages.push(CopyRom)
    }

    // check if the repo is cloned
    if !base_dir.join(&spec.repo.name).exists() {
        needed_stages.push(CloneRepo)
    }

    // check if the build script exists
    if !base_dir.join("build.sh").exists() {
        needed_stages.push(CreateBuildScript)
    }

    // return
    needed_stages
}

impl ToString for Region {
    fn to_string(&self) -> String {
        use Region::*;

        let retval = match self {
            US => "us",
            EU => "eu",
            JP => "jp",
        };

        retval.to_owned()
    }
}
