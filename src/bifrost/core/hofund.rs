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
    use std::path::PathBuf;

    #[test]
    fn test_read_failure() {
        assert!(read(Path::new("not@çtu@llyar34lp4th")).is_err());
    }

    #[test]
    fn test_read_success() {
        let from = PathBuf::from("tests")
            .join("test_user")
            .join("test_app_dir")
            .join("Bifrost.toml");
        assert!(read(from.as_ref()).is_ok());
    }

    #[test]
    fn test_write() {
        let path = PathBuf::from("tests")
            .join("test_user")
            .join("test_app_dir")
            .join("test_write.txt");

        assert!(write(&path, "testing... 1.. 2.. 3..".as_bytes()).is_ok());
        remove_file(&path).expect("`test_write` failed to `hofund::remove_file`");
    }

    #[test]
    fn test_append() {
        let path = PathBuf::from("tests")
            .join("test_user")
            .join("test_app_dir")
            .join("test_append.txt");

        let bytes = "testing... 1.. 2.. 3..\n".as_bytes();
        write(&path, &bytes).expect("`test_append` failed to `hofund::write`");

        assert!(append(&path, &bytes).is_ok());

        let left = "testing... 1.. 2.. 3..\ntesting... 1.. 2.. 3..\n".as_bytes();
        let middle = read(&path).expect("`test_append` failed to `hofund::read`");
        let right = middle.as_bytes();
        assert_eq!(left, right);
        remove_file(&path).expect("`test_append` failed to `hofund::remove` test file");
    }

    #[test]
    fn test_read() {
        let path = PathBuf::from("tests")
            .join("test_user")
            .join("test_app_dir")
            .join("test_read.txt");

        let left = "testing... 1.. 2.. 3..\n".as_bytes();
        write(&path, &left).expect("`test_read` failed to `hofund::write`");

        let right = read(&path);
        assert!(right.is_ok());
        assert_eq!(left, right.unwrap().as_bytes());

        remove_file(&path).expect("`test_read` failed to `hofund::read`");
        assert!(read(&path).is_err());
    }

    #[test]
    fn test_remove_file() {
        let path = PathBuf::from("tests")
            .join("test_user")
            .join("test_app_dir")
            .join("test_remove_file.txt");
        let bytes = "buffalo buffalo buffalo buffalo".as_bytes();
        write(&path, &bytes).expect("`test_remove_file` failed to `hofund::write`");
        assert!(remove_file(&path).is_ok());
        assert!(read(&path).is_err());
    }
}
