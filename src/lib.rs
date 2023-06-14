pub mod builder;
pub mod error;
pub mod types;

pub mod prelude;

use std::{fs, os::unix::prelude::PermissionsExt, path::Path};

use error::SmbuilderError;
use prelude::Makeopt;

pub fn get_makeopts_string(makeopts: &[Makeopt]) -> String {
    let mut retval = String::from("");

    for opt in makeopts.iter() {
        retval.push_str(format!("{}={} ", opt.key, opt.value).as_str());
    }

    retval
}

pub fn make_file_executable(path: &Path) -> Result<(), SmbuilderError> {
    let file_metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(e) => {
            return Err(SmbuilderError::new(
                Some(Box::new(e)),
                format!(
                    "failed to get the metadata of the file at {}",
                    &path.display()
                ),
            ))
        }
    };

    match fs::set_permissions(
        path,
        fs::Permissions::from_mode(
            file_metadata.permissions().mode() + 0o111, // equivalent of a chmod +x.
        ),
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(SmbuilderError::new(
            Some(Box::new(e)),
            format!(
                "failed to set permissions on the file at {}",
                &path.display(),
            ),
        )),
    }
}
