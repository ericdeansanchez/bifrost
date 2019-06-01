//! Primary structures, mehtods, and functions that support `bifrost::ops`.
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use crate::core::config::{values_of, Config};
use crate::core::working_dir::WorkingDir;
use crate::util::BifrostResult;
use crate::ArgMatches;

/// Primary structure which `bifrost::ops operate upon.
///
/// A `WorkSpace` is composed of the following:
/// * name - the name of a realm's `WorkSpace`.
/// * mode - describes how `contents` of this `WorkSpace` should be handled.
/// * config - a [`Config`](struct.Config.html)
/// * contents - an `Option<Vec<WorkingDir>>`, see [`WorkingDir`](struct.WorkingDir.html)
#[derive(Debug)]
pub struct WorkSpace {
    // name of workspace that helps keep track of multiple `WorkSpace`'s.
    name: Option<String>,
    // mode describing _how_ and/or _which_ operations should be performed.
    mode: Mode,
    // configuration information that describes the current Bifrost realm.
    config: Config,
    // contents of the `WorkSpace` in the form of `WorkingDir`s.
    contents: Option<Vec<WorkingDir>>,
    // a `BifrostPath` which is a light wrapper around a `PathBuf`.
    bifrost_path: Option<BifrostPath>,
}

/// A default `WorkSpace` is constructed with a default `Config` in `Mode::Normal`.
/// All other fields are `None`.
impl Default for WorkSpace {
    fn default() -> Self {
        WorkSpace {
            name: None,
            mode: Mode::Normal,
            config: Config::default(),
            contents: None,
            bifrost_path: None,
        }
    }
}

impl WorkSpace {
    /// Initializes a `WorkSpace` from a `Config` and `clap::ArgMatches`. If no
    /// contents were specified, then `ws_args` will be empty. When `ws_args` is
    /// empty the default behavior is to construct this `WorkSpace`'s contents from
    /// the current working directory.
    ///
    /// If arguments were passed (i.e. through `$ bifrost load --contents main.c`),
    /// then all arguments present in `ws_args` are processed. Valid path-names
    /// are converted to `PathBuf`s from which this `WorkSpace`'s contents will
    /// be constructed.
    pub fn init(config: Config, args: &ArgMatches) -> Self {
        let (ws_mode, ws_args) = WorkSpaceBuilder::values_from_args(&args);
        let (ws_name, ws_ignore_list) = WorkSpaceBuilder::values_from_config(&config);

        let mut contents: Vec<WorkingDir> = Vec::new();
        if ws_args.is_empty() {
            contents.push(WorkingDir::new(config.cwd()).ignore(&ws_ignore_list));
        } else {
            let paths = WorkSpaceBuilder::args_to_paths(&config, ws_args);

            if paths.is_empty() {
                eprintln!("error: contents were passed but no arguments represent valid content");
                process::exit(1);
            }

            for path in paths {
                contents.push(WorkingDir::new(path).ignore(&ws_ignore_list));
            }
        }

        WorkSpace {
            name: Some(ws_name),
            mode: ws_mode,
            config,
            contents: Some(contents),
            bifrost_path: None,
        }
    }
}

struct WorkSpaceBuilder;

impl WorkSpaceBuilder {
    // Returns the ignore list from the manifest if it exists. Otherwise, an
    // empty vector will be returned meaning that no files will be ignored in
    // the `WorkSpace`.
    fn build_workspace_ignore_list(config: &Config) -> Vec<String> {
        match config
            .manifest()
            .and_then(|m| m.workspace_config().and_then(|ws| ws.ignore()))
        {
            Some(list) => list.to_owned(),
            None => Vec::new(),
        }
    }

    // Returns the name of the workspace if it exists; otherwise, the workspace
    // name is derived from the current working directory's top-level directory.
    fn get_workspace_name(config: &Config) -> String {
        const DEFAULT_TOML_NAME: &str = "name of workspace";

        fn name_from_cwd(cwd: &PathBuf) -> String {
            match cwd.file_name() {
                Some(name) => String::from(
                    name.to_str()
                        .expect("error: could not `unwrap` name from cwd"),
                ),
                None => panic!("error: could not determine name from cwd from `file_name`"),
            }
        }
        match config
            .manifest()
            .and_then(|m| m.workspace_config().and_then(|ws| ws.name()))
        {
            None => name_from_cwd(config.cwd()),
            Some(name) => {
                if name == DEFAULT_TOML_NAME {
                    return name_from_cwd(config.cwd());
                } else {
                    return String::from(name);
                }
            }
        }
    }

    // Convenience function that gets the `Mode` flag and the `contents` from
    // command line arguments.
    fn values_from_args(args: &ArgMatches) -> (Mode, Vec<String>) {
        (
            flag(&args),
            values_of("contents", &args).map_or(vec![], |v| v),
        )
    }

    // Convenience function that gets the name of the workspace and the `ignore_list`
    // from a `Config`.
    fn values_from_config(config: &Config) -> (String, Vec<String>) {
        (
            WorkSpaceBuilder::get_workspace_name(&config),
            WorkSpaceBuilder::build_workspace_ignore_list(&config),
        )
    }

    // Convenience function that `strip`s trailing slashes from command line arguments,
    // `check`s if the current working directory has paths that contain these arguments,
    // and then returns absolute-path-versions of these arguments.
    // More validation may need to be done; this is a naive implementation.
    fn args_to_paths(config: &Config, args: Vec<String>) -> Vec<PathBuf> {
        let ws_path_names = strip(&args);
        let ws_paths = check(config.cwd(), ws_path_names);
        return to_abs_path(config.cwd(), ws_paths);
    }
}

#[derive(Copy, Clone, Debug)]
enum Mode {
    Auto,
    Modified,
    Normal,
}

// Collects `names` that can be considered valid paths.
fn check<P>(cwd: P, names: Vec<String>) -> Vec<String>
where
    P: AsRef<Path>,
{
    names
        .into_iter()
        .filter(|n| is_valid(&cwd, n).unwrap_or(false))
        .collect()
}

// A `name` is valid if it exists within the current Bifrost realm.
fn is_valid<P>(cwd: P, name: &str) -> BifrostResult<bool>
where
    P: AsRef<Path>,
{
    for entry in fs::read_dir(cwd)? {
        let entry = entry?;
        if let Some(path) = entry.path().to_str() {
            if path.contains(name) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

// Returns `names` without trailing slashes.
fn strip(names: &Vec<String>) -> Vec<String> {
    let mut names_without_trailing_slashes: Vec<String> = Vec::new();
    for n in names {
        names_without_trailing_slashes.push(String::from(strip_trailing_slash(n)));
    }
    names_without_trailing_slashes
}

// Returns a `name` without any trailing slashes.
fn strip_trailing_slash(name: &str) -> &str {
    if name.len() > 1 && name.ends_with("\\") || name.ends_with("/") {
        return &name[..name.len() - 1];
    }
    name
}

// Converts `name`d strings to `PathBuf`s if and only if the name is contained
// within a `PathBuf` that resides in the current working directory.
fn to_abs_path<P>(cwd: P, paths: Vec<String>) -> Vec<PathBuf>
where
    P: AsRef<Path>,
{
    let mut abs_paths: Vec<PathBuf> = Vec::new();
    if let Ok(read_dir) = fs::read_dir(cwd) {
        for entry in read_dir {
            if let Ok(e) = entry {
                if let Some(s) = e.path().to_str() {
                    if paths.iter().any(|p| **p == *s || s.contains(p)) {
                        abs_paths.push(e.path().to_path_buf());
                    }
                }
            }
        }
    }
    abs_paths
}

// Returns the `Mode` flag.
fn flag(args: &ArgMatches) -> Mode {
    if args.is_present("auto") {
        return Mode::Auto;
    } else if args.is_present("modified") {
        return Mode::Modified;
    }
    Mode::Normal
}

/// A light wrapper around a `PathBuf`. This structure exists to deter improper
/// command execution. That is, `bifrost::ops` require target paths to be `BifrostPath`s
/// in order to prevent operations from being performed on arbitrary `PathBuf`s.
///
/// For example, it should not be possible for a user to remove their entire
/// home directory (or another arbitrary directory). Moreover, the goal of `BifrostPath`
/// is to validate and enforce invariants on paths that `load`, `unload`, and
/// `show` will use as targets.
#[derive(Debug)]
pub struct BifrostPath {
    path: PathBuf,
}
