//! Abstractions over `std::fs` and  `walkdir::WalkDir`.
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::env;
use std::fmt;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use crate::util::{BifrostPath, BifrostResult, OperationInfo};

extern crate dirs;
extern crate walkdir;

use walkdir::{DirEntry, WalkDir};

/// The primary data structure for walking and structuring working-directory information.
pub struct WorkingDir {
    /// The `absolute path to the root`'s parent directory.
    parent: Option<PathBuf>,
    /// The absolute path to this working directory.
    root: PathBuf,
    /// This working directory's sub directories ordered by depth with respect
    /// to the `root`.
    dirs: BinaryHeap<DirEntryExt>,
    /// Absolute paths to this working directory's files.
    files: Vec<PathBuf>,
    /// Names of files and/or directories to `ignore` when
    /// `walk`ing this working directory.
    ignore_list: HashSet<String>,
    /// The size of all files in this `WorkingDir`.
    size: u64,
}

impl Default for WorkingDir {
    fn default() -> Self {
        let root = env::current_dir()
            .expect("error: could not construct default `WorkingDir` with `env::current_dir`");
        let parent = root.parent().map(|p| p.to_path_buf());

        WorkingDir {
            parent,
            root,
            dirs: BinaryHeap::new(),
            files: vec![],
            ignore_list: HashSet::new(),
            size: 0,
        }
    }
}

impl fmt::Debug for WorkingDir {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WorkingDir")
            .field("parent", &self.parent)
            .field("root", &self.root)
            .field("dirs", &self.dirs)
            .field("files", &self.files)
            .field("ignore_list", &self.ignore_list)
            .finish()
    }
}

impl WorkingDir {
    /// Constructs a new `WorkingDir` given an absolute path to a directory or
    /// file. This path becomes the `root` of this `WorkingDir`. The `parent` of this
    /// path is stored as well. The remaining fields are initialized with empty
    /// collections.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let root = path.as_ref().to_path_buf();
        let parent = root.parent().map(|p| p.to_path_buf());

        WorkingDir {
            root,
            parent,
            ..Default::default()
        }
    }

    /// Stores a list of names to ignore for subsequent access. The actual filtering
    /// does not occur until [`walk`](struct.WorkingDir.html#method.walk) is called.
    pub fn ignore<S>(mut self, list: &Vec<S>) -> Self
    where
        S: AsRef<str>,
    {
        for path in list {
            self.ignore_list.insert(path.as_ref().to_string());
        }
        self
    }

    /// Walks a directory (or file) beginning from the `root`. Utilizes the
    /// `WalkDir` iterator and `self.ignore_list` so that walks will not descend
    /// into unwanted directories (or dir-entries).
    ///
    /// The entries are handled in two different ways:
    ///
    /// ## Directories
    ///
    /// Directories are pushed onto a `BinaryHeap`; this heap a min-heap that
    /// utilizes a directory's depth for ordering. This is so that when, for example,
    /// this `WorkingDir` is copied/written to the Bifrost container, no recursion
    /// occurs. Contents are written essentially in level-order so that there's no
    /// attempt to create a child-directory before it's parent.
    ///
    /// ## Files
    ///
    /// Absolute paths, in the form of `PathBuf`s are pushed onto a vector.
    pub fn walk(mut self) -> BifrostResult<Self> {
        let root = &self.root;
        let ignore_list = &self.ignore_list;
        let walkdir = WalkDir::new(root)
            .into_iter()
            .filter_entry(|e| !ignorable(e, ignore_list));

        for entry in walkdir {
            if let Ok(entry) = entry {
                if entry.path().is_dir() {
                    self.dirs.push(DirEntryExt(entry));
                } else if entry.path().is_file() {
                    // Record the 'incoming' file sizes.
                    self.size += fs::metadata(entry.path())
                        .expect("error: `WorkingDir::walk` failed to unwrap `fs::metadata`")
                        .len();

                    self.files.push(entry.into_path());
                }
            }
        }
        Ok(self)
    }

    /// Returns the `root` of this working directory.
    pub fn root(&self) -> &Path {
        self.root.as_path()
    }

    /// Returns the optional `parent` of this work directory.
    pub fn parent(&self) -> Option<&PathBuf> {
        self.parent.as_ref()
    }

    /// Returns the size of the `WorkingDir`.
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Returns a mutable reference to this `WorkingDir`'s underlying `dirs`.
    /// This allows the `load` operation to avoid manual heap-traversal.
    pub fn dirs_as_mut(&mut self) -> &mut BinaryHeap<DirEntryExt> {
        &mut self.dirs
    }

    /// Tries to load this `WorkingDir` into the container to the target destination
    /// specified by the `BifrostPath`.
    pub fn load(&mut self, bifrost_path: BifrostPath) -> BifrostResult<OperationInfo> {
        Ok(try_load(self, bifrost_path)?)
    }
}

/// Filter function used to filter entries in the construction of the `WalkDir` iterator.
fn ignorable(entry: &DirEntry, ignore_list: &HashSet<String>) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| ignore_list.iter().any(|a| s.starts_with(a)))
        .unwrap_or(false)
}

/// Light-ish wrapper around a `walkdir::DirEntry`. It is used to get access
/// to an entry's depth. Since Rust's `BinaryHeap` is a max-heap, the ordering
/// on this type needs to be inverted to get min-heap behavior out of this collection.
#[derive(Debug)]
pub struct DirEntryExt(DirEntry);

impl Eq for DirEntryExt {}

impl Ord for DirEntryExt {
    fn cmp(&self, other: &DirEntryExt) -> Ordering {
        other.0.depth().cmp(&self.0.depth())
    }
}

impl PartialOrd for DirEntryExt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.depth().partial_cmp(&self.0.depth())
    }
}

impl PartialEq for DirEntryExt {
    fn eq(&self, other: &Self) -> bool {
        self.0.depth() == other.0.depth()
    }
}

/// Tries to load the specified `WorkingDir` to the target destination specified by the `BifrostPath`.
fn try_load(wd: &mut WorkingDir, bifrost_path: BifrostPath) -> BifrostResult<OperationInfo> {
    let bytes = _load(wd, &bifrost_path)?;

    // If the number of `load`ed bytes differs from the number of `walk`ed bytes, stop.
    if bytes != wd.size {
        failure::bail!("error: `workingdir::try_load` failed to load entire `WorkingDir`");
    }

    Ok(OperationInfo {
        bytes: Some(bytes),
        name: String::from("default"),
        text: None,
        ..Default::default()
    })
}

/// Loads the specified `WorkingDir` to the target destination specified by the `BifrostPath` by:
///
/// * loading the top-level-directory, then
/// * loading the remaining directories, and then
/// * loading the files
///
/// Returns a `BifrostResult` containing the number of bytes written (or errors).
fn _load(wd: &mut WorkingDir, to: &BifrostPath) -> BifrostResult<u64> {
    let pb = wd
        .parent()
        .expect("BUG: `WorkingDir::parent` should not be `None` here")
        .clone();
    let parent = pb.to_str().expect(
        "BUG: `bifrost_load::_load` failed to convert `WorkingDir::parent` `PathBuf` `to_str`",
    );

    // `dirs` will be empty after this if block.
    if let Some(from) = wd.dirs_as_mut().pop() {
        _load_top_level_dir(&parent, &from, &to)?;

        while let Some(from) = wd.dirs_as_mut().pop() {
            _load_dir(&parent, &from, &to)?;
        }
    } else {
        // There are no directories to load, so create the target `to` path
        // to begin loading the file(s).
        fs::create_dir_all(&to.path)?;
    }
    return _load_files(&parent, &wd, &to);
}

/// Creates the top-level directory at its target destination. To construct the appropriate
/// suffix to be adjoined to the `BifrostPath` the `parent` is stripped from the path of the
/// dir-entry. The directory is created if and only if a destination suffix was able to be
/// constructed.
///
/// # Example
/// If the intent is to load `src`, then the process is conceptually as follows:
///
/// * from: /Users/username/project/src
/// * to: /Users/username/.bifrost/container/project
/// * parent: "/Users/username/project"
/// * destination_suffix: "src"
/// * target: /Users/username/.bifrost/container/project/src
///
/// # Note
/// The example above is only meant to demonstrate the reasoning behind stripping
/// prefixes from paths. It does not necessary indicate the actual behavior produced
/// due to the nature of naming v.s. not-naming workspaces.
fn _load_top_level_dir(parent: &str, from: &DirEntryExt, to: &BifrostPath) -> BifrostResult<()> {
    let to_dir = from.0.path().clone();
    if let Ok(destination_suffix) = to_dir.strip_prefix(parent) {
        fs::create_dir_all(&to.path.join(destination_suffix))?;
        return Ok(());
    } else {
        failure::bail!(
            "error: `workingdir::_load_top_level_dir` failed to construct destination target"
        );
    }
}

/// Creates directories within the Bifrost container.
fn _load_dir(parent: &str, from: &DirEntryExt, to: &BifrostPath) -> BifrostResult<()> {
    let to_dir = from.0.path().clone();
    if let Ok(to_dir) = to_dir.strip_prefix(parent) {
        fs::create_dir_all(&to.path.join(to_dir))?;
    }
    Ok(())
}

/// Creates files within the Bifrost container and copies the contents from the
/// workspace source files to the destination files created in the Bifrost container.
fn _load_files(parent: &str, wd: &WorkingDir, to: &BifrostPath) -> BifrostResult<u64> {
    let mut nbytes = 0;
    for entry in &wd.files {
        if let Ok(to) = entry.strip_prefix(parent).map(|e| to.path.join(e)) {
            File::create(&to)?;
            if let Ok(n) = fs::copy(entry, to) {
                nbytes += n;
            }
        }
    }
    Ok(nbytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_working_dir() {
        // If this call succeeds then,
        let wd = WorkingDir::default();
        // this call to unwrap should succeed
        // as well. Otherwise, the former panics.
        let cwd = env::current_dir().unwrap();

        assert_eq!(cwd, wd.root);
        assert!(wd.dirs.is_empty());
        assert!(wd.files.is_empty());
        assert!(wd.ignore_list.is_empty());
    }

    #[test]
    fn test_walk_dirs<'a>() {
        let test_dir: [&str; 7] = [
            "./tests/test_dir/file.txt",
            "./tests/test_dir/file.cpp",
            "./tests/test_dir/test_dir_nested/test_nested_ignore.md",
            "./tests/test_dir/test_dir_nested/nested_file.txt",
            "./tests/test_dir/test_ignore_file.txt",
            "./tests/test_dir/file.c",
            "./tests/test_dir/file.rs",
        ];

        let mut test_set: HashSet<&'a str> = HashSet::new();

        for path in test_dir.iter() {
            test_set.insert(path);
        }

        if let Ok(wd) = WorkingDir::new("./tests/test_dir").walk() {
            for path in wd.files.iter() {
                if let Some(path) = path.to_str() {
                    assert!(test_set.contains(path));
                }
            }
        }
    }

    #[test]
    fn test_walk_dirs_with_ignores<'a>() {
        let test_dir: [&str; 5] = [
            "./tests/test_dir/file.txt",
            "./tests/test_dir/file.cpp",
            "./tests/test_dir/test_dir_nested/nested_file.txt",
            "./tests/test_dir/file.c",
            "./tests/test_dir/file.rs",
        ];

        let list = vec![
            String::from("test_nested_ignore.md"),
            String::from("test_ignore_file.txt"),
            String::from("test_ignore_dir"),
        ];

        let mut test_set: HashSet<&'a str> = HashSet::new();

        for path in test_dir.iter() {
            test_set.insert(path);
        }

        if let Ok(wd) = WorkingDir::new("./tests/test_dir").ignore(&list).walk() {
            for path in wd.files.iter() {
                if let Some(path) = path.to_str() {
                    assert!(test_set.contains(path));
                }
            }
        }
    }

    #[test]
    fn test_propose_target_suffix() -> BifrostResult<()> {
        Ok(())
    }
}
