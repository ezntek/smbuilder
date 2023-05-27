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
pub mod build_select;

use cursive::{traits::*, views, CursiveRunnable};

pub enum Action {
    Play,
    Edit,
    Info,
}

use Action::*;

trait SmbuilderUiView {
    fn setup_ui(&self) -> Box<dyn View>;
}

#[derive(Default)]
pub struct App {}

impl App {
    // ui related setup functions
    fn setup_global_callbacks(&self, s: &mut CursiveRunnable) {
        s.add_global_callback('q', |s| s.quit())
    }

    fn setup_uis(&self, s: &mut CursiveRunnable) {}

    // nice public functions
    pub fn new() -> App {
        App::default()
    }

    pub fn run(&self) {
        let mut siv = cursive::default();

        // run some setup functions
        self.setup_global_callbacks(&mut siv);
        self.setup_uis(&mut siv);

        // run
        siv.run();
    }
}
