use std::fs;
use std::path::{Path, PathBuf};

use failure;

pub type BifrostResult<T> = failure::Fallible<T>;

/// A light wrapper around a `PathBuf`. This structure exists to deter improper
/// command execution. That is, `bifrost::ops` require target paths to be `BifrostPath`s
/// in order to prevent operations from being performed on arbitrary `PathBuf`s.
///
/// For example, it should not be possible for a user to remove (e.g. `unload`) their entire
/// home directory (or another arbitrary directory). Moreover, the goal of `BifrostPath`
/// is to validate and enforce invariants on paths that `load`, `unload`, and
/// `show` use as targets.
#[derive(Debug)]
pub struct BifrostPath {
    pub path: PathBuf,
}

impl Clone for BifrostPath {
    fn clone(&self) -> Self {
        BifrostPath {
            path: self.path.to_path_buf(),
        }
    }
}

/// Wrapper around a `PathBuf` used to construct paths that enforce the following invariants:
///
/// * the path does not contain black-listed names
/// * the path is unique meaning that it does not currently exist within the Bifrost container
impl BifrostPath {
    /// Constructs a new `BifrostPath` from a home directory path and a workspace's
    /// name.
    pub fn new(home_path: &Path, name: Option<&String>) -> BifrostResult<Self> {
        // Check that the name is not blacklisted.
        BifrostPath::check(name)?;

        // Construct path to the Bifrost container.
        let container_path = home_path.join(".bifrost").join("container");
        // Construct path to this workspace's destination within the Bifrost container.
        let path =
            container_path.join(name.expect("BUG: `WorkSpace::name` should not be `None` here"));

        // Bail if the path already exists.
        if fs::metadata(&path).is_ok() {
            failure::bail!("error: the proposed path {:#?} already exists", path);
        }

        Ok(BifrostPath { path })
    }

    /// Checks that the `proposed_name` does not contain any black-listed names.
    fn check(proposed_name: Option<&String>) -> BifrostResult<()> {
        let black_list: Vec<&str> = vec![".bifrost", "container", ".bifrost_config", "tmp"];

        if let Some(name) = proposed_name {
            if black_list.iter().any(|b| *b == name) {
                failure::bail!("error: proposed path contains blacklisted name(s)");
            }
        } else {
            failure::bail!("error: proposed path cannot be empty or `None`");
        }

        // The `path` checks good!
        Ok(())
    }
}

pub struct OperationInfo {
    pub bytes: Option<u64>,
    pub name: String,
}

#[macro_use]
mod macros;

pub mod error;
pub mod template;
