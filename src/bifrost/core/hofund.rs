//! # Primary read and write utilities for Bifrost realms.
//!
//! ## Hofund
//! Hofund is the sword Heimdallr uses to open the Bifrost.
//! Here, it is the name of a module that was hard to name. Hofund, is more fun
//! than `util` or `bifrost_io` or something like that.
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;

use crate::util::BifrostResult;

/// Write the contents to the given `path`.
pub fn write(path: &Path, contents: &[u8]) -> BifrostResult<()> {
    let mut f = File::create(path)?;
    f.write_all(contents)?;
    Ok(())
}

/// Writes the contents to the given `path` in "append mode".
pub fn append(path: &Path, contents: &[u8]) -> BifrostResult<()> {
    let mut f = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)?;
    f.write_all(contents)?;
    Ok(())
}

/// Removes a single file specified by `path`.
pub fn remove_file(path: &Path) -> BifrostResult<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) => failure::bail!("error: could not remove file due to {}", e),
    }
}

/// Reads a file from the given `path`.
pub fn read(path: &Path) -> BifrostResult<String> {
    match fs::read_to_string(path) {
        Ok(s) => Ok(s),
        Err(e) => failure::bail!(
            "error: could not read from path `{}` due to `{}`",
            path.display(),
            e
        ),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_failure() {
        assert!(read(Path::new("not@çtu@llyar34lp4th")).is_err());
    }

    #[test]
    fn test_read_success() {
        assert!(read(Path::new("tests/test_user/test_app_dir/Bifrost.toml")).is_ok());
    }
}
