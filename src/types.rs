use crate::{error::Error, get_makeopts_string};
use derive_builder::Builder;
use std::fmt::Debug;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Region {
    #[default]
    US,
    EU,
    JP,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Rom {
    pub region: Region,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Repo {
    pub name: String,
    pub url: String,
    pub branch: String,
    pub supports_packs: bool,
    pub supports_textures: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Makeopt {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Datapack {
    pub label: String,
    pub path: PathBuf,
    pub enabled: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TexturePack {
    pub path: PathBuf,
    pub enabled: bool,
}

#[derive(Default, Builder, Debug, Deserialize, Serialize)]
#[builder(setter(into))]
pub struct Spec {
    pub rom: Rom,
    pub repo: Repo,
    pub jobs: Option<u8>,
    pub name: Option<String>,
    pub additional_makeopts: Option<Vec<Makeopt>>,
    pub texture_pack: Option<TexturePack>,
    pub packs: Option<Vec<Datapack>>,
}

impl Spec {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Spec, Error> {
        let file_string = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => return Err(Error::new(Some(Box::new(e)), "Failed to read the file")),
        };

        let retval = match serde_yaml::from_str::<Spec>(&file_string) {
            Ok(s) => s,
            Err(e) => {
                return Err(Error::new(
                    Some(Box::new(e)),
                    "Failed to parse the file into a yaml",
                ))
            }
        };

        Ok(retval)
    }

    pub fn get_build_script(&self, repo_path: &Path) -> String {
        let makeopts_string = if let Some(makeopts) = &self.additional_makeopts {
            get_makeopts_string(makeopts)
        } else {
            String::new()
        };

        let jobs = if let Some(j) = self.jobs { j } else { 2 };

        format!(
            "
#!/bin/sh

# Script Generated by smbuilder.
# DO NOT EDIT; YOUR CHANGES
# WILL NOT BE SAVED.

make -C {} {} -j{}
        ",
            repo_path.display(),
            makeopts_string,
            jobs
        )
    }
}
