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

use serde_derive;

#[cfg(test)]
mod tests {}

#[derive(Debug, PartialEq, Eq, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RenderAPI {
    GL,
    GLLegacy,
    D3D11,
    D3D12,
}


#[derive(Debug, PartialEq, Eq, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Sm64exCoopRenderAPI {
    GL,
    GLLegacy,
    D3D11,
    D3D12,
    Dummy,
}

#[derive(Debug, PartialEq, Eq, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Render96WindowAPI {
    SDL2,
    DXGI,
}

#[derive(Debug, PartialEq, Eq, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Sm64exWindowAPI {
    SDL1,
    SDL2,
    DXGI,
}

#[derive(Debug, PartialEq, Eq, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Sm64exCoopWindowAPI {
    SDL1,
    SDL2,
    DXGI,
}

#[derive(Debug, PartialEq, Eq, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Sm64exAudioAPI {
    SDL1,
    SDL2,
}

#[derive(Debug, PartialEq, Eq, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Sm64exCoopAudioAPI {
    SDL1,
    SDL2,
    Dummy,
}

#[derive(Debug, PartialEq, Eq, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Sm64exControllerAPI {
    SDL1,
    SDL2,
}

#[derive(Debug, PartialEq, Eq, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(tag = "opt", content = "arg")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Render96exMakeopts {
    Debug,
    Compare,
    NonMatching,
    TargetRpi,
    TargetWeb,
    #[serde(rename = "OSX_BUILD")]
    OSXBuild,
    TargetArch(String),
    TargetBits(u8),
    #[serde(rename = "BETTERCAMERA")]
    BetterCamera,
    #[serde(rename = "NODRAWINGDISTANCE")]
    NoDrawingDistance,
    TextureFix,
    #[serde(rename = "EXT_OPTIONS_MENU")]
    ExtendedOptionsMenu,
    #[serde(rename = "TEXTSAVES")]
    TextSaves,
    ExternalData,
    #[serde(rename = "DISCORDRPC")]
    DiscordRichPresence,
    RenderApi(RenderAPI),
    WindowApi(Render96WindowAPI),
    #[serde(rename = "RMODERN")]
    ModernRendering,
    WindowsBuild,
}

#[derive(Debug, PartialEq, Eq, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(tag = "opt", content = "arg")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Sm64exMakeopts {
    Debug,
    Compare,
    NonMatching,
    TargetRpi,
    TargetWeb,
    #[serde(rename = "OSX_BUILD")]
    OSXBuild,
    NoPie,
    TargetArch(String),
    TargetBits(u8),
    #[serde(rename = "BETTERCAMERA")]
    BetterCamera,
    #[serde(rename = "NODRAWINGDISTANCE")]
    NoDrawingDistance,
    TextureFix,
    #[serde(rename = "EXT_OPTIONS_MENU")]
    ExtendedOptionsMenu,
    #[serde(rename = "TEXTSAVES")]
    TextSaves,
    ExternalData,
    #[serde(rename = "DISCORDRPC")]
    DiscordRichPresence,
    RenderApi(RenderAPI),
    WindowApi(Sm64exWindowAPI),
    AudioApi(Sm64exAudioAPI),
    ControllerApi(Sm64exControllerAPI),
    #[serde(rename = "RMODERN")]
    ModernRendering,
    WindowsBuild,
}

#[derive(Debug, PartialEq, Eq, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(tag = "opt", content = "arg")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Sm64exCoopMakeopts {
    Debug,
    Development,
    TargetN64, 
    TargetRpi,
    TargetWeb,
    #[serde(rename = "OSX_BUILD")]
    OSXBuild,
    TargetArch(String),
    TargetBits(u8),
    #[serde(rename = "IMMEDIATELOAD")]
    ImmediateLoad,
    #[serde(rename = "BETTERCAMERA")]
    BetterCamera,
    #[serde(rename = "NODRAWINGDISTANCE")]
    NoDrawingDistance,
    TextureFix,
    #[serde(rename = "EXT_OPTIONS_MENU")]
    ExtendedOptionsMenu,
    #[serde(rename = "TEXTSAVES")]
    TextSaves,
    ExternalData,
    #[serde(rename = "DISCORDRPC")]
    DiscordRichPresence,
    DiscordSdk,
    #[serde(rename = "DOCKERBUILD")]
    DockerBuild,
    OptLevel(i8),
    DebugInfoLevel(u8),
    Profile,
    Headless,
    Icon,
    #[serde(rename = "USE_APP")]
    UseMacOSApp,
    RenderApi(Sm64exCoopRenderAPI),
    WindowApi(Sm64exCoopWindowAPI),
    AudioApi(Sm64exCoopAudioAPI),
    #[serde(rename = "RMODERN")]
    ModernRendering,
    WindowsBuild,
}
pub trait MakeoptsType {
    fn get_defaults() -> Vec<Self> where
        Self: Sized;
}

impl MakeoptsType for Render96exMakeopts {
    fn get_defaults() -> Vec<Self>
    where
        Self: Sized
    {
        Vec::from([
            Render96exMakeopts::Compare,
            Render96exMakeopts::NonMatching,
            Render96exMakeopts::TargetArch(String::from("native")),
            Render96exMakeopts::TargetBits(0),
            Render96exMakeopts::ExtendedOptionsMenu,
            Render96exMakeopts::RenderApi(RenderAPI::GL),
            Render96exMakeopts::WindowApi(Render96WindowAPI::SDL2),
        ])
    }
}
impl MakeoptsType for Sm64exMakeopts {
    fn get_defaults() -> Vec<Self>
    where
        Self: Sized
    {
        Vec::from([
            Sm64exMakeopts::Compare,
            Sm64exMakeopts::NonMatching,
            Sm64exMakeopts::NoPie,
            Sm64exMakeopts::TargetArch(String::from("native")),
            Sm64exMakeopts::TargetBits(0),
            Sm64exMakeopts::ExtendedOptionsMenu,
            Sm64exMakeopts::RenderApi(RenderAPI::GL),
            Sm64exMakeopts::WindowApi(Sm64exWindowAPI::SDL2),
            Sm64exMakeopts::AudioApi(Sm64exAudioAPI::SDL2),
            Sm64exMakeopts::ControllerApi(Sm64exControllerAPI::SDL2),
        ])
    }
}

impl MakeoptsType for Sm64exCoopMakeopts {
    fn get_defaults() -> Vec<Self>
    where
        Self: Sized
    {    
        Vec::from([
            Sm64exCoopMakeopts::ImmediateLoad,
            Sm64exCoopMakeopts::BetterCamera,
            Sm64exCoopMakeopts::NoDrawingDistance,
            Sm64exCoopMakeopts::ExtendedOptionsMenu,
            Sm64exCoopMakeopts::DiscordSdk,
            Sm64exCoopMakeopts::OptLevel(-1),
            Sm64exCoopMakeopts::DebugInfoLevel(2),
            Sm64exCoopMakeopts::Icon,
            Sm64exCoopMakeopts::UseMacOSApp,
        ])
    }
}