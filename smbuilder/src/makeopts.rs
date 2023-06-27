use crate::{makeopt, prelude::*};

#[allow(missing_docs)]

/// A common set of make flags
/// across all sm64ex-based
/// builds. Useful for launchers
/// who need checkboxes.
pub enum BaseMakeopts {
    BetterCamera,
    NoDrawingDistance,
    TextureFix,
    ExtOptionsMenu,
    TextSaves,
    ExternalData,
    DiscordPresence,
}

#[allow(missing_docs)]

/// An extension of `BaseMakeopts`
/// but with sm64ex-coop specific
/// options.
pub enum CoopMakeopts {
    Base(BaseMakeopts),
    ImmediateLoad,
    DiscordInviteSupport,
}

impl From<BaseMakeopts> for Makeopt {
    fn from(value: BaseMakeopts) -> Makeopt {
        use BaseMakeopts::*;

        match value {
            BetterCamera => makeopt!("BETTERCAMERA", "1"),
            NoDrawingDistance => makeopt!("NODRAWINGDISTANCE", "1"),
            TextureFix => makeopt!("TEXTURE_FIX", "1"),
            ExtOptionsMenu => makeopt!("EXT_OPTIONS_MENU", "1"),
            TextSaves => makeopt!("TEXTSAVES", "1"),
            ExternalData => makeopt!("EXTERNAL_DATA", "1"),
            DiscordPresence => makeopt!("DISCORDRPC", "1"),
        }
    }
}

impl From<CoopMakeopts> for Makeopt {
    fn from(value: CoopMakeopts) -> Self {
        use CoopMakeopts::*;

        match value {
            Base(b) => b.into(),
            ImmediateLoad => makeopt!("IMMEDIATELOAD", "1"),
            DiscordInviteSupport => makeopt!("DISCORD_SDK", "1"),
        }
    }
}
