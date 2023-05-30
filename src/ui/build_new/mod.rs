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

mod repo_select;

use super::*;
use crate::builder::Repo;
use cursive::views::*;

#[derive(Default, Copy, Clone)]
enum SpecSelectionStage {
    #[default]
    Repo,
    Rom,
    Packs,
    Name,
    CompilerOptions,
}

impl SpecSelectionStage {
    fn prev(slf: &Self) -> Option<Self> {
        use SpecSelectionStage::*;

        match slf {
            Repo => None,
            Rom => Some(Repo),
            Packs => Some(Rom),
            Name => Some(Packs),
            CompilerOptions => Some(Name),
        }
    }

    fn next(slf: &Self) -> Option<Self> {
        use SpecSelectionStage::*;

        match slf {
            Repo => Some(Rom),
            Rom => Some(Packs),
            Packs => Some(Name),
            Name => Some(CompilerOptions),
            CompilerOptions => None,
        }
    }
}

enum BuildNewDialogAction {
    Quit,
    Back,
    Next,
}
