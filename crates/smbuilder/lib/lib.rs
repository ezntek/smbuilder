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
pub mod prelude;
pub mod builder;
pub mod common;
pub mod makeopts;

use std::path::Path;

pub fn init() {
    // Set some dirs for readability
    let home_dir = std::env!("HOME");
    let smbuilder_data_dir = Path::new(&home_dir).join(".local/share/smbuilder");
    let smbuilder_config_dir = Path::new(&home_dir).join(".local/share/smbuilder");

    // Create the directories
    for dir in vec![smbuilder_data_dir, smbuilder_config_dir].iter() {
        std::fs::create_dir(&dir)
            .expect(format!("Failed to create {}! Perhaps the directory already exists?", dir.display()).as_str());        
    }
}