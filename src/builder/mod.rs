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

mod builder;
mod types;

use std::{fs, os::unix::prelude::PermissionsExt, path::PathBuf};

pub use builder::*;
pub use types::*;

fn make_file_executable(path: &PathBuf) -> Result<(), String> {
    let file_metadata = match fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(e) => return Err(
            format!("failed to get the metadata of the file: {} at path {}",
                    e.to_string(),
                    &path.display())
        )
    };

    match fs::set_permissions(
        &path,
    fs::Permissions::from_mode(
            file_metadata.permissions().mode() + 0o111 // equivalent of a chmod +x.
        )
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(
            format!("failed to set permissions on the file: {}",
                    &path.display())
        )
    }
}