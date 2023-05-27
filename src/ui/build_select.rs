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

struct BuildSelectView {
    builds: HashMap<String, Spec>
}

fn ui_play_build(s: &mut Cursive) {}
fn ui_edit_build(s: &mut Cursive) {}
fn ui_info_build(s: &mut Cursive) {}
fn ui_new_build(s: &mut Cursive) {}
fn ui_del_build(s: &mut Cursive) {}

impl BuildSelectView {
    fn populate_builds(&mut self, builds: Vec<Spec>) {
        for build in builds {
            self.builds.insert(build.name.clone(), build);
        }
    }
}

impl SmbuilderUiView for BuildSelectView {
    fn setup_ui(&self) -> Box<dyn View> {
        let select_view = SelectView::<String>::new()
            .with_name("select build");
        
        let resizable_selectview = ResizedView::with_fixed_size(
            (10, 5),
            select_view
        );

        let side_btns = LinearLayout::vertical()
            .child(
                Button::new("Play", |s| ui_play_build(s))
            )
            .child(
                Button::new("Edit", |s| ui_edit_build(s))
            )
            .child(
                Button::new("Info", |s| ui_info_build(s))
            )
            .child(
                Button::new("Delete", |s| ui_del_build(s))
            )
            .child(
                DummyView // cheap spacer
            )
            .child(
                Button::new("New", |s| ui_new_build(s))
            );

        Box::new(resizable_selectview)
    }
}