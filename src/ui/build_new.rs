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
use crate::builder::Repo;
use cursive::views::*;

fn build_new_state_manager__emit_action(action: BuildNewDialogAction) {}

#[derive(Default, Copy, Clone)]
enum SpecSelectionStage {
    #[default]
    Repo,
    Rom,
    Packs,
    Name,
    CompilerOptions,
}

enum BuildNewDialogAction {
    Quit,
    Back,
    Next,
}

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

impl SmbuilderUiView for RepoSelectDialog {
    fn setup_ui(&self) -> Box<dyn View> {
        let scrollable_select_view = SelectView::<String>::new().scrollable();

        Box::new(DummyView)
    }
}

impl StateManagedSmbuilderDialog<SpecSelectionStage> for RepoSelectDialog {
    fn setup_dlg(&self, mgr: &mut DialogsStateManager<SpecSelectionStage>) -> Box<Dialog> {
        let dlg = Dialog::around(self.setup_ui())
            .button("Quit", |s| {
                let _ = s.pop_layer(); // pop the dialog off and discarding the dialog
            })
            .button("Back", |_s| mgr.prev())
            .button("Next", |_s| mgr.next());

        Box::new(dlg)
    }
}
