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

use std::collections::HashMap;

use super::*;
use crate::builder::Spec;
use cursive::{views::*, Cursive};

#[derive(Default)]
pub struct BuildSelectView {
    builds: HashMap<String, Spec>,
}

fn ui_buildselect_play_build(s: &mut Cursive) {}
fn ui_buildselect_edit_build(s: &mut Cursive) {}
fn ui_buildselect_info_build(s: &mut Cursive) {}
fn ui_buildselect_new_build(s: &mut Cursive) {}
fn ui_buildselect_del_build(s: &mut Cursive) {}

impl BuildSelectView {
    pub fn new() -> Self {
        BuildSelectView::default()
    }

    pub fn populate_builds(&mut self, builds: Vec<Spec>) {
        for build in builds {
            self.builds.insert(build.name.clone(), build);
        }
    }
}

impl SmbuilderUiView for BuildSelectView {
    fn setup_ui(&self) -> Box<dyn View> {
        let mut select_view = SelectView::<String>::new();

        let builds_keys = self.builds.keys();
        if builds_keys.len() == 0 {
            select_view.add_item_str("There are no builds!")
        } else {
            select_view.add_all_str(self.builds.keys());
        }
        let scroll_selectview = select_view.scrollable();

        let resizable_selectview = ResizedView::with_max_size((20, 10), scroll_selectview);

        let side_btns = LinearLayout::vertical()
            .child(Button::new("Play", ui_buildselect_play_build))
            .child(Button::new("Edit", ui_buildselect_edit_build))
            .child(Button::new("Info", ui_buildselect_info_build))
            .child(Button::new("Delete", ui_buildselect_del_build))
            .child(
                DummyView.min_height(2), // cheap spacer
            )
            .child(Button::new("New", ui_buildselect_new_build));

        let view = LinearLayout::horizontal()
            .child(resizable_selectview)
            .child(DummyView.min_width(4))
            .child(side_btns);

        Box::new(view)
    }
}
