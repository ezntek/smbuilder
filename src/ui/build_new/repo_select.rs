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
use super::*;

#[derive(Default)]
pub struct RepoSelectDialog {
    repos: Vec<Repo>,
    selected_repo: Option<Repo>,
}

impl RepoSelectDialog {
    pub fn new() -> Self {
        RepoSelectDialog::default()
    }

    pub fn populate_repos() {}
}

impl SmbuilderDialog for RepoSelectDialog {
    fn setup_dlg(&self) -> Box<Dialog> {
        let dlg = Dialog::new()
            .button("Quit", |s| {
                s.pop_layer();
            })
            .button("Back", |_s| todo!())
            .button("Next", |_s| todo!());

        Box::new(dlg)
    }
}

impl From<SpecSelectionStage> for Option<RepoSelectDialog> {
    fn from(val: SpecSelectionStage) -> Self {
        use SpecSelectionStage::*;

        match val {
            Repo => Some(RepoSelectDialog::new()),
            _ => None,
        }
    }
}
