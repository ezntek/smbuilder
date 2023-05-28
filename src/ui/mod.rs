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

pub mod build_new;
pub mod build_select;

use cursive::{
    event::Event,
    traits::*,
    views::{self, Dialog, DummyView},
    CursiveRunnable,
};

pub enum Action {
    Play,
    Edit,
    Info,
}

use Action::*;

use self::build_select::BuildSelectView;

trait SmbuilderUiView {
    fn setup_ui(&self) -> Box<dyn View> {
        Box::new(DummyView)
    }
}

trait SmbuilderDialog {
    fn setup_dlg(&self) -> Box<Dialog> {
        Box::new(Dialog::around(DummyView))
    }
}

trait StateManagedSmbuilderDialog<T>
where
    T: Clone + Copy,
{
    #[allow(unused_variables)]
    fn setup_dlg(&self, mgr: &mut DialogsStateManager<T>) -> Box<Dialog> {
        Box::new(Dialog::around(DummyView))
    }
}

pub struct DialogsStateManager<T>
where
    T: Clone + Copy,
{
    curr_state: (usize, T),
    states_order: Vec<T>,
}

impl<T> DialogsStateManager<T>
where
    T: Clone + Copy,
{
    pub fn new(default: T, default_idx: usize, order: Vec<T>) -> DialogsStateManager<T> {
        DialogsStateManager {
            curr_state: (default_idx, default),
            states_order: order,
        }
    }

    pub fn prev(&mut self) {
        let new_idx = self.curr_state.0 - 1;
        self.curr_state = (new_idx, self.states_order[new_idx])
    }

    pub fn next(&mut self) {
        let new_idx = self.curr_state.0 + 1;
        self.curr_state = (new_idx, self.states_order[new_idx])
    }

    pub fn set_state(&mut self, new_idx: usize) {
        self.curr_state = (new_idx, self.states_order[new_idx])
    }

    pub fn get_state_idx(&self) -> usize {
        self.curr_state.0
    }

    pub fn get_state_value(&self) -> T {
        self.curr_state.1
    }
}

#[derive(Default)]
pub struct App {}

impl App {
    // ui related setup functions
    fn setup_global_callbacks(&self, s: &mut CursiveRunnable) {
        s.add_global_callback('q', |s| s.quit());
        s.add_global_callback(Event::Key(cursive::event::Key::Esc), |s| s.quit());
    }

    fn setup_uis(&self, s: &mut CursiveRunnable) {
        let view = BuildSelectView::default().setup_ui();
        let dlg = Dialog::around(view).title("smbuilder");

        s.add_layer(dlg);
    }

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
