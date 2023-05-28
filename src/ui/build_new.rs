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
use crate::builder::{Datapack, Repo, Spec, TexturePack};
use cursive::{views::*, Cursive};

#[derive(Default)]
pub struct NameTexturePackSelectDialog {
    name: String,
    texture_pack: TexturePack,
}

impl NameTexturePackSelectDialog {
    pub fn new() -> Self {
        NameTexturePackSelectDialog::default()
    }
}

impl SmbuilderUiView for NameTexturePackSelectDialog {
    fn setup_ui(&self) -> Box<dyn View> {
        // custom name row
        let custom_name_editview = EditView::new();
        let custom_name_label = TextView::new("(OPTIONAL) give it a name: ");

        let custom_name_row = LinearLayout::horizontal()
            .child(custom_name_label)
            .child(custom_name_editview);

        // texture packs row

        let view = LinearLayout::vertical().child(custom_name_row);

        Box::new(view)
    }
}
