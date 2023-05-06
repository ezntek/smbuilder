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

fn make_file_executable(path: PathBuf) {
    let file_permissions = fs::metadata(&path)
        .expect("getting the metadata of the file failed!")
        .permissions();

    fs::set_permissions(
        &path,
        fs::Permissions::from_mode(
            file_permissions.mode() + 0o111 // this is the equivalent of a chmod +x.
        )
    ).expect(format!("Setting the file permissions from {} to {} failed!", file_permissions.mode(), file_permissions.mode()+0o111).as_str())
}