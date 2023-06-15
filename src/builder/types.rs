#[derive(Debug)]
pub enum SmbuilderSetupStage {
    WriteSpec,
    CloneRepo,
    CopyRom,
    CreateBuildScript,
}

impl ToString for SmbuilderSetupStage {
    fn to_string(&self) -> String {
        use SmbuilderSetupStage::*;

        let result = match self {
            WriteSpec => "write the spec to disk",
            CloneRepo => "clone the repository",
            CopyRom => "copy the base ROM",
            CreateBuildScript => "create the build script",
        };

        result.to_owned()
    }
}
