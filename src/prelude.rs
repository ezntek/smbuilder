/// Re-exports from the `n64romconvert` crate,
/// as its conversion features are (somewhat)
/// critical to the build process.
pub mod romconvert {
    pub use n64romconvert::{byte_endian_swap, byte_swap, endian_swap, RomType};
}

pub use crate::builder::*;
pub use crate::types::*;
pub use crate::SmbuilderError;
pub use crate::{get_makeopts_string, make_file_executable, BuilderCallbacks, LogType};
pub use romconvert::*;
