//! [InProgress]
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

type VisitorMut = Fn(PathBuf, &mut WorkingDir);

/// Primary structure to contain paths from directories that
/// have been walked.
#[derive(Debug)]
pub struct WorkingDir {
    pub root: PathBuf,
    pub paths: Vec<PathBuf>,
    pub ignore_list: HashSet<PathBuf>,
}

impl WorkingDir {
    /// Initializes a `WorkingDir`. If the `cwd` is passed then use that as the
    /// `WorkingDir.root`. Otherwise, attempt to use the `current_dir`.
    ///
    /// Specifies the ignore-`list`, if applicable, and walks the directory
    /// accordingly.
    pub fn init(cwd: Option<PathBuf>, list: Vec<PathBuf>) -> WorkingDir {
        let mut root: PathBuf;

        if let Some(cwd) = cwd {
            root = cwd.clone();
        } else {
            root = env::current_dir().expect("could not initialize root directory");
        }

        let mut frost_dir = WorkingDir {
            root: root.clone(),
            paths: vec![],
            ignore_list: HashSet::new(),
        };

        if !list.is_empty() {
            frost_dir.ignore(list);
        }

        // Consider using this result in future error handling...
        let _result = walk_dirs(&root, &WorkingDir::stash, &mut frost_dir)
            .expect("an error occurred while walking the directory");

        frost_dir
    }

    /// Stashes paths in a `FrostDir`'s by pushing them onto its path vector.
    fn stash(path: PathBuf, content: &mut WorkingDir) {
        content.paths.push(path);
    }

    fn ignore(&mut self, list: Vec<PathBuf>) {
        for path in list {
            self.ignore_list.insert(path);
        }
    }
}

/// Recursively walks the directory specified by `dir`. Skip any path that `ends_with` a value
/// that exists in the `ignore_list` and apply `visit` to any file encountered that meets this
/// criteria.
pub fn walk_dirs(dir: &PathBuf, visit: &VisitorMut, content: &mut WorkingDir) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() && !content.ignore_list.iter().any(|a| path.ends_with(a)) {
                walk_dirs(&path, visit, content)?;
            } else if path.is_file() && !content.ignore_list.iter().any(|a| path.ends_with(a)) {
                visit(path, content);
            }
        }
    }

    Ok(())
}

///  If `prefix` is not `None`, build a `PathBuf` from a `PathBuf` and a `&str`.
pub fn path_builder(prefix: Option<PathBuf>, suffix: &str) -> Option<PathBuf> {
    if let Some(path) = prefix {
        return Some(path.join(suffix));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_walk_dirs_empty_ignore() {
        let test_dir: [PathBuf; 7] = [
            PathBuf::from("./tests/test_dir/file.txt"),
            PathBuf::from("./tests/test_dir/file.cpp"),
            PathBuf::from("./tests/test_dir/test_dir_nested/test_nested_ignore.md"),
            PathBuf::from("./tests/test_dir/test_dir_nested/nested_file.txt"),
            PathBuf::from("./tests/test_dir/test_ignore_file.txt"),
            PathBuf::from("./tests/test_dir/file.c"),
            PathBuf::from("./tests/test_dir/file.rs"),
        ];

        let mut test_set: HashSet<&PathBuf> = HashSet::new();

        for path in test_dir.iter() {
            test_set.insert(path);
        }

        let d = WorkingDir::init(Some(PathBuf::from("./tests/test_dir")), Vec::new());

        for path in d.paths.iter() {
            assert!(test_set.contains(path));
        }
    }

    #[test]
    fn test_walk_dirs_basic_full_ignore_path() {
        let test_dir: [PathBuf; 5] = [
            PathBuf::from("./tests/test_dir/file.txt"),
            PathBuf::from("./tests/test_dir/file.cpp"),
            PathBuf::from("./tests/test_dir/test_dir_nested/nested_file.txt"),
            PathBuf::from("./tests/test_dir/file.c"),
            PathBuf::from("./tests/test_dir/file.rs"),
        ];

        let ignore_list: Vec<PathBuf> = vec![
            PathBuf::from("./tests/test_dir/test_dir_nested/test_nested_ignore.md"),
            PathBuf::from("./tests/test_dir/test_ignore_file.txt"),
            PathBuf::from("./tests/test_dir/test_ignore_dir"),
        ];

        let mut test_set: HashSet<&PathBuf> = HashSet::new();

        for path in test_dir.iter() {
            test_set.insert(path);
        }

        let d = WorkingDir::init(Some(PathBuf::from("./tests/test_dir")), ignore_list);

        for path in d.paths.iter() {
            assert!(test_set.contains(path));
        }
    }

    #[test]
    fn test_walk_dirs_basic_short_ignore_path() {
        let test_dir: [PathBuf; 5] = [
            PathBuf::from("./tests/test_dir/file.txt"),
            PathBuf::from("./tests/test_dir/file.cpp"),
            PathBuf::from("./tests/test_dir/test_dir_nested/nested_file.txt"),
            PathBuf::from("./tests/test_dir/file.c"),
            PathBuf::from("./tests/test_dir/file.rs"),
        ];

        let ignore_list: Vec<PathBuf> = vec![
            PathBuf::from("test_nested_ignore.md"),
            PathBuf::from("test_ignore_file.txt"),
            PathBuf::from("test_ignore_dir"),
        ];

        let mut test_set: HashSet<&PathBuf> = HashSet::new();

        for path in test_dir.iter() {
            test_set.insert(path);
        }

        let d = WorkingDir::init(Some(PathBuf::from("./tests/test_dir")), ignore_list);

        for path in d.paths.iter() {
            assert!(test_set.contains(path));
        }
    }
}
