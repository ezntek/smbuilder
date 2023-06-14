#[derive(Debug)]
pub enum SmbuilderSetupStage {
    WriteSpec,
    CloneRepo,
    CopyRom,
    CreateBuildScript,
}
