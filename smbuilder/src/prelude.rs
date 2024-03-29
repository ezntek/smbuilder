/// Re-export functions from the `n64romconvert` crate.
pub mod romconvert {
    pub use n64romconvert::{byte_endian_swap, byte_swap, determine_format, endian_swap, RomType};
}

// Builder stuff
pub use crate::builder::builder::Builder;
pub use crate::builder::types as builder_types;

// callbacks
pub use crate::callbacks::types as callback_types;
pub use crate::callbacks::*;

// spec
pub use crate::spec::*;

// core types
pub use crate::types::*;

// errors
pub use crate::error::macros as error_macros;
pub use crate::error::{Error, ErrorCause};
pub use error_macros::err;

// other stuff
pub use romconvert::*;
