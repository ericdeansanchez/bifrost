//! Abstractions over `std::fs` and  `walkdir::WalkDir`.
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::env;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::util::BifrostResult;

extern crate dirs;
extern crate walkdir;

use walkdir::{DirEntry, WalkDir};

/// # The primary data structure for walking and structuring working-directory information.
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
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let root = path.as_ref().to_path_buf();
        let parent = root.parent().map(|p| p.to_path_buf());
        WorkingDir {
            root,
            parent,
            ..Default::default()
        }
    }

    pub fn ignore(mut self, list: &[&str]) -> Self {
        for path in list {
            self.ignore_list.insert(path.to_string());
        }
        self
    }

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
                    self.files.push(entry.into_path());
                }
            }
        }
        Ok(self)
    }
}

#[derive(Debug)]
struct DirEntryExt(DirEntry);

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

fn ignorable(entry: &DirEntry, ignore_list: &HashSet<String>) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| ignore_list.iter().any(|a| s.starts_with(a)))
        .unwrap_or(false)
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
            "test_nested_ignore.md",
            "test_ignore_file.txt",
            "test_ignore_dir",
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
}
