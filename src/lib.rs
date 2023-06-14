pub mod build;
pub mod error;
pub mod types;

pub mod prelude;

use std::{fs, os::unix::prelude::PermissionsExt, path::Path};

pub fn make_file_executable(path: &Path) -> Result<(), String> {
    let file_metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(e) => {
            return Err(format!(
                "failed to get the metadata of the file: {} at path {}",
                e,
                &path.display()
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
        Err(e) => Err(format!(
            "failed to set permissions on the file at {}: {}",
            &path.display(),
            e
        )),
    }
}
