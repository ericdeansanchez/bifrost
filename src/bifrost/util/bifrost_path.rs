use crate::util::BifrostResult;

use std::fs;
use std::path::{Path, PathBuf};

const BIFROST: &str = ".bifrost";
const CONTAINER: &str = "container";

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
        let container_path = home_path.join(BIFROST).join(CONTAINER);
        // Construct path to this workspace's destination within the Bifrost container.
        let path =
            container_path.join(name.expect("BUG: `WorkSpace::name` should not be `None` here"));

        // Bail if the path already exists.
        if fs::metadata(&path).is_ok() {
            failure::bail!("error: the proposed path {:#?} already exists", path);
        }

        Ok(BifrostPath { path })
    }

    pub fn try_from_existing<P>(home_path: P, name: Option<&String>) -> BifrostResult<Self>
    where
        P: AsRef<Path>,
    {
        if let Ok(bifrost_path) = BifrostPath::from_existing(home_path, name) {
            return Ok(bifrost_path);
        } else {
            // from_existing failed do other stuff
        }
        Ok(BifrostPath {
            path: PathBuf::new(),
        })
    }

    fn from_existing<P>(home_path: P, name: Option<&String>) -> BifrostResult<Self>
    where
        P: AsRef<Path>,
    {
        let container_path = home_path.as_ref().join(BIFROST).join(CONTAINER);
        let path =
            container_path.join(name.expect("BUG: `WorkSpace::name` should not be `None` here"));

        if fs::metadata(&path).is_err() {
            failure::bail!(
                "error: `BifrostPath::from_existing` cannot construct an existing path from `WorkSpace::name`"
            );
        }

        Ok(BifrostPath { path })
    }

    /// Checks that the `proposed_name` does not contain any black-listed names.
    fn check(proposed_name: Option<&String>) -> BifrostResult<()> {
        let black_list: Vec<&str> = vec![BIFROST, CONTAINER, ".bifrost_config", "tmp"];

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

pub fn get_path_or_panic(maybe_path: Option<BifrostPath>) -> PathBuf {
    maybe_path
        .expect(
            "BUG: `bifrost_unload::unload` failed because it expected `BifrostPath` to be `Some`",
        )
        .path
}

pub fn handle_bad_path(path: PathBuf) -> String {
    pub fn handle_str(s: &str) -> String {
        if s.is_empty() {
            return String::from("empty path");
        }
        String::from("bad string")
    }

    match path.to_str() {
        Some(s) => handle_str(s),
        None => String::from("None"),
    }
}
