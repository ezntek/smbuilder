// Copyright 2023 Eason Qin <eason@ezntek.com>.
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use serde_derive;
use std::fs;
use std::path::{PathBuf, Path};

use crate::makeopts::MakeoptsType;

#[cfg(test)]
mod tests {}

fn get_dummy_base_path() -> PathBuf {
    Path::new(std::env!("HOME")).join(".local/share/smbuilder")
}

pub fn get_toml_makeopts_from_string<S>(string: S) -> String
where
    S: AsRef<str>
     + ToString
{
    let s = string.to_string();
    let makeopt_expressions = s.split(" ").collect::<Vec<&str>>();
    let mut toml_string = String::from("[");
    
    for expr in makeopt_expressions {
        let makeopt = expr.split("=").collect::<Vec<&str>>();
        toml_string.push_str(format!("{{opt={},arg={}}},", makeopt[0], makeopt[1]).as_str());
    };
    toml_string.push_str("]");
    toml_string
}

pub fn get_enum_makeopts_from_string<S, M>(string: S) -> Vec<M>
where
    S: AsRef<str>
     + ToString,
    M: MakeoptsType
     + for<'a> serde::Deserialize<'a>
{
    let toml_makeopts = get_toml_makeopts_from_string(string);

    #[derive(serde_derive::Deserialize)]
    struct DummyStruct<M> {
        makeopts: Vec<M>
    }

    toml::from_str::<DummyStruct<M>>(toml_makeopts.as_str())
        .expect("One makeopt in the string may not be a valid makeopt! Please check your spelling and try again.")
        .makeopts
}

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Region {
    US,
    EU,
    JP,
    SH
}

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Debug, PartialEq, Eq)]
pub struct Rom {
    pub region: Region,
    pub path: PathBuf,
}

impl Rom {
    pub fn default() -> Rom {
        Rom {
            path: get_dummy_base_path(),
            region: Region::US,
        }
    }

    pub fn new(region: Region, path: PathBuf) -> Rom {
        Rom {
            region,
            path
        }
    }
}

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Debug)]
pub struct Repo {
    pub name: String,
    pub url: String,
    pub branch: String,
    pub supports_packs: bool,
    pub supports_textures: bool
}

impl Repo {
    pub fn default() -> Repo {
        Repo {
            name: "dummy repo".to_string(),
            url: "https://github.com/ezntek/smbuilder".to_string(),
            branch: "main".to_string(),
            supports_packs: false,
            supports_textures: false
        }
    }
}

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Debug)]
pub struct DynOSPack {
    pub enabled: bool,
    pub label: String,
    pub path: PathBuf,
}

/// The Build Specification Structure. Contains all the metadata required to run the Smbuilder class and the SmbuilderBuilder class, etc.
///
/// Supports:
//
/// * Jobs (jobs = [make] -jX)
/// * Name (A custom name can be used, else it is repo.name)
/// * Additional Make Options (eg. FOO=1 BAR=baz QUUX=0, make  FOO=1 BAR=baz QUUX=0 -jX)
/// * A custom texture pack (think Render96)
/// * DynOS data packs (also think Render96, but other ports like sm64ex-coop supports them too)
///
#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct BuildSpec<M: MakeoptsType> {
    /// The number of jobs to be put together with the MAKEOPTS during the compile stage.
    pub jobs: u8,
    /// The name of the build, it will default to the name of the repo if left empty.
    pub name: String,
    /// Any additional makeopts to be added to the make call. Will include the jobs.
    pub additional_makeopts: Vec<M>,
    /// The executable path. Not playable if empty, playable if not empty.
    pub executable_path: Option<String>,
    /// A custom texture pack (There can only be one!)
    pub texture_pack_path: Option<PathBuf>,
    /// The repo struct
    pub repo: Repo,
    /// The rom struct
    pub rom: Rom,
    /// Any DynOS packs the user wishes to add
    pub dynos_packs: Option<Vec<DynOSPack>>,
}

impl<M> BuildSpec<M>
where
    M: MakeoptsType
        + for<'a> serde::Deserialize<'a>
        + serde::Serialize
{
    pub fn from_file<P: AsRef<Path>>(path: P) -> BuildSpec<M>{
        let toml_str = fs::read_to_string(path).unwrap();
        let s = toml::from_str::<TomlSpec<M>>(&toml_str).unwrap();
        let mut spec = s.build_settings;
        spec.dynos_packs = s.dynos_packs;
        spec
    }

    pub fn get_makeopts_string(&self, string_makeopts: Option<String>) -> String {
        // dirty hacks to get the string names of the enums
        #[derive(serde_derive::Deserialize)]
        struct Makeopt {
            opt: String,
            arg: Option<String>,
        }

        #[derive(serde_derive::Serialize)]
        struct DummyStruct<'a, M: MakeoptsType> {
            makeopts: &'a Vec<M>,
        }

        // initialize the string
        let mut retval = String::new();

        let temp_string = toml::to_string(
            &DummyStruct {
                makeopts: &self.additional_makeopts
            }).unwrap();
        
        let vec_makeopts: Vec<Makeopt> = toml::from_str(&temp_string).unwrap();
        
        for makeopt in vec_makeopts {
            if let Some(arg) = makeopt.arg {
                retval.push_str(&format!("{}={}", makeopt.opt, arg))
            } else {
                retval.push_str(&format!("{}=1", makeopt.opt))
            }
        };

        if let Some(x) = string_makeopts {
            retval.push_str(&x);
        }

        retval
    }
    
    pub fn generate_build_script<P>(&self, repo_path: P, string_makeopts: Option<String>) -> String
    where
        P: AsRef<P> + std::fmt::Display
    {
        format!("#!/bin/sh\ncd {}\nmake {} -j{}", repo_path, &self.get_makeopts_string(string_makeopts), &self.jobs)
    }
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct TomlSpec<M: MakeoptsType> {
    pub build_settings: BuildSpec<M>,
    pub dynos_packs: Option<Vec<DynOSPack>>,
}

impl<M> Into<BuildSpec<M>> for TomlSpec<M>
where
    M: MakeoptsType
        + serde::Serialize
        + for<'a> serde::Deserialize<'a>
{
    fn into(self) -> BuildSpec<M> {
        let spec = self.build_settings; // Make an easier to type shortcut because I'm lazy lol

        BuildSpec {
            jobs: spec.jobs,
            name: spec.name,
            additional_makeopts: spec.additional_makeopts,
            executable_path: spec.executable_path,
            texture_pack_path: spec.texture_pack_path,
            repo: spec.repo,
            rom: spec.rom,
            dynos_packs: self.dynos_packs
        }
    }
}

impl<M> From<BuildSpec<M>> for TomlSpec<M>
where
    M: MakeoptsType
        + serde::Serialize
        + for<'a> serde::Deserialize<'a>
{
    fn from(value: BuildSpec<M>) -> Self {
        TomlSpec {
            dynos_packs: value.dynos_packs,
            build_settings: BuildSpec { // XXX: psst! This is to avoid implementing the copy trait on DynOSPack, because it makes use of a string type.
                jobs: value.jobs,
                name: value.name,
                additional_makeopts: value.additional_makeopts,
                executable_path: value.executable_path,
                texture_pack_path: value.texture_pack_path,
                repo: value.repo,
                rom: value.rom,
                dynos_packs: None,
            },
        }
    }
}