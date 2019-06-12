//! Primary structures, mehtods, and functions that facilitate `bifrost::ops`.
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use crate::core::config::{self, Config};
use crate::core::workingdir::WorkingDir;
use crate::util::{BifrostPath, BifrostResult, OperationInfo};
use crate::ArgMatches;

/// Primary structure which `bifrost::ops operate upon.
///
/// A `WorkSpace` is composed of the following:
/// * name - the name of a Bifrost realm's `WorkSpace`.
/// * mode - describes _how_ `contents` of this `WorkSpace` should be handled.
/// * config - a [`Config`](struct.Config.html)
/// * contents - an `Option<Vec<WorkingDir>>`, see [`WorkingDir`](struct.WorkingDir.html)
#[derive(Debug)]
pub struct WorkSpace {
    // Name of workspace that helps keep track of multiple `WorkSpace`'s.
    name: Option<String>,
    // Mode describing _how_ operations should be performed.
    mode: Mode,
    // Configuration information that describes the current Bifrost realm.
    config: Config,
    // Contents of the `WorkSpace` in the form of `WorkingDir`s.
    contents: Option<Vec<WorkingDir>>,
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
        }
    }
}

impl WorkSpace {
    /// Returns the name of a workspace.
    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    /// Returns the `Mode` of a workspace.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Returns the `Config` of a workspace.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Returns the contents of a workspace.
    pub fn contents(&self) -> Option<&Vec<WorkingDir>> {
        self.contents.as_ref()
    }

    /// Constructs a `LoadSpace`.
    pub fn to_load_space(config: Config, args: &ArgMatches) -> LoadSpace {
        WorkSpaceArgs::parse(config, &args).to_load_space()
    }
}

impl AsMut<WorkSpace> for WorkSpace {
    fn as_mut(&mut self) -> &mut WorkSpace {
        self
    }
}

/// The `Mode` specifiecs _how_ operations should be performed. A `WorkSpace`'s
/// contents will be handled differently based on this value (e.g. if `--auto` has
/// been specified then changes to source files will be detected and be automatically
/// re-`load`ed).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Mode {
    Auto,
    Modified,
    Normal,
}

/// This structure is similar to `WorkSpace`; however, there are a few differences.
/// Namely, `ignore_list` and `contents`. The `ignore_list` does not appear in `WorkSpace`
/// and `contents` is a optional `Vec<String>` instead of a `Vec<WorkingDir>`.
///
/// The reason for this is `WorkSpaceArgs` takes values from command line arguments
/// in the form of `ArgMatches` and from the `Config`. The value(s) of `contents` when
/// passed from the command line may not be appropriate values to construct `WorkingDir`s.
///
/// The `contents` are kept as strings until they have been processed to a point where
/// they are okay to use; processing may not be able to produce valid `contents`. In this
/// case, it is best to avoid constructing anything that is `BifrostOperable` or whose
/// owner is `BifrostOperable`.
///
/// New `WorkingDir`s take ownership of a copy of the `ignore_list` when they are constructed.
/// In previous iterations `WorkSpace`s owned an `ignore_list`. This way eliminates this one
/// small duplication.
struct WorkSpaceArgs {
    name: Option<String>,
    mode: Mode,
    config: Config,
    contents: Option<Vec<String>>,
    ignore_list: Vec<String>,
}

impl WorkSpaceArgs {
    /// Gets the mode, contents, name, and ignore_list from `ArgMatches` and a `Config`.
    fn parse(config: Config, args: &ArgMatches) -> WorkSpaceArgs {
        let (ws_mode, ws_contents) = WorkSpaceBuilder::values_from_args(&args);
        let (ws_name, ws_ignore_list) = WorkSpaceBuilder::values_from_config(&config);

        WorkSpaceArgs {
            name: Some(ws_name),
            mode: ws_mode,
            config,
            contents: Some(ws_contents),
            ignore_list: ws_ignore_list,
        }
    }

    /// Constructs a `LoadSpace` for `WorkSpaceArgs`. `WorkSpaceArgs` has all the
    /// information needed to construct a `WorkSpace`. The _proposed_ `contents`
    /// are passed off to `WorkSpaceBuilder::args_to_paths` for some _basic_
    /// validation/manipulation.
    ///
    /// If no paths were returned, then there is nothing to construct `WorkingDir`s from
    /// and no reason to construct the `WorkSpace`.
    ///
    /// Otherwise, a new `WorkSpace` is constructed with new `contents` of type `Vec<WorkingDir>`
    /// and a `LoadSpace` is returned.
    fn to_load_space(self) -> LoadSpace {
        let mut workding_dirs: Vec<WorkingDir> = Vec::new();
        if let Some(contents) = self.contents {
            if contents.is_empty() {
                workding_dirs.push(WorkingDir::new(self.config.cwd()).ignore(&self.ignore_list));
            } else {
                let paths = WorkSpaceBuilder::args_to_paths(&self.config, contents, &is_loadable);
                if paths.is_empty() {
                    eprintln!(
                        "error: contents were passed but no arguments represent \
                         valid current working directory entries"
                    );
                    process::exit(1);
                }

                for path in paths {
                    workding_dirs.push(WorkingDir::new(path).ignore(&self.ignore_list));
                }
            }
        }

        LoadSpace {
            workspace: WorkSpace {
                name: self.name,
                mode: self.mode,
                config: self.config,
                contents: Some(workding_dirs),
            },
            bifrost_path: None,
        }
    }
}

pub trait BifrostOperable {
    /// Prepares implementor to be built.
    fn prep(&mut self) -> BifrostResult<&mut BifrostOperable>;
    /// Builds implementor to be executed.
    fn build(&mut self) -> BifrostResult<&mut BifrostOperable>;
    /// Executes implementor's executive function(s).
    fn exec(&mut self) -> BifrostResult<OperationInfo>;
}

#[derive(Debug)]
pub struct BaseSpace {}

impl BifrostOperable for BaseSpace {
    fn prep(&mut self) -> BifrostResult<&mut BifrostOperable> {
        unimplemented!();
    }

    fn build(&mut self) -> BifrostResult<&mut BifrostOperable> {
        unimplemented!();
    }

    fn exec(&mut self) -> BifrostResult<OperationInfo> {
        unimplemented!();
    }
}

/// Primary data structure used to `load` `WorkSpace`s into the Bifrost container.
#[derive(Debug)]
pub struct LoadSpace {
    /// The `WorkSpace` to be loaded.
    workspace: WorkSpace,
    /// The target path to `load` the `WorkSpace` to.
    bifrost_path: Option<BifrostPath>,
}

/// A `LoadSpace`'s primary goal is to `load` contents into the Bifrost container.
/// However, it also implements helper methods that make accessing its nested
/// structure a little more ergonomic at this level.
impl LoadSpace {
    /// Returns a reference to the underlying `WorkSpace`.
    pub fn workspace(&self) -> &WorkSpace {
        &self.workspace
    }

    /// Returns a reference to the underlying `home_path` `PathBuf` defined
    /// upon configuration.
    pub fn home_path(&self) -> &PathBuf {
        self.workspace.config().home_path()
    }

    /// Returns a optional reference to the underlying `WorkSpace` name.
    pub fn name(&self) -> Option<&String> {
        self.workspace.name()
    }

    /// Returns a `clone`d version of the target `BifrostPath`.
    pub fn bifrost_path(&self) -> Option<BifrostPath> {
        self.bifrost_path.clone()
    }

    /// Returns the underlying `WorkSpace`'s contents `as_mut`.
    pub fn contents_as_mut(&mut self) -> Option<&mut Vec<WorkingDir>> {
        self.workspace.contents.as_mut()
    }

    /// Returns the underlying `WorkSpace` `as_mut`.
    pub fn workspace_as_mut(&mut self) -> &mut WorkSpace {
        self.workspace.as_mut()
    }

    /// Loads this spaces's `WorkSpace` to the target `BifrostPath`.
    ///
    /// # Panics
    ///
    /// * if the call to [`prep`] has failed or
    /// * if it cannot unwrap a `WorkSpace::name`
    ///
    /// It is possible that this method **panics**, however, this means
    /// that this method was called prior to a call to [`prep`] or that
    /// the call to [`prep`] failed.
    ///
    /// It might be possible for this method to attempt to unwrap a `WorkSpace::name`
    /// that is `None`. This seems **unlikely**, however, this can be ruled out if
    /// it can be shown that `WorkSpace::default` is the only operation that
    /// produces a `WorkSpace` with a name that is `None`.
    pub fn load(&mut self) -> BifrostResult<OperationInfo> {
        let mut nbytes = 0u64;
        let path = self
            .bifrost_path()
            .expect("BUG: `BifrostPath` should not be `None` here");

        // The contents must be mutable because loading a `WorkingDir`
        // mutates its underlying `dirs` field.
        if let Some(contents) = self.contents_as_mut() {
            for wd in contents {
                let info = wd.load(path.clone())?;
                if let Some(n) = info.bytes {
                    nbytes += n;
                }
            }
        }

        // The name really should not be `None` at this point.
        let name = self
            .name()
            .expect("BUG: `LoadSpace::load` failed to unwrap `WorkSpace::name`")
            .clone();

        // Returns the sum of bytes written and the name of the workspace they
        // were written from.
        Ok(OperationInfo {
            bytes: Some(nbytes),
            name,
        })
    }
}

/// Implements `BifrostOperable` for `LoadSpace`.
/// A `LoadSpace` is `prep`-able, `build`-able, and `exec`-utable. These methods
/// follow a builder-esq pattern. They are meant to be called in sequence (chained).
/// Both `prep` and `build` return `BifrostResult`s containing structures that implement
/// `BifrostOperable` (or errors). Conversely, `build` is a consuming operation that
/// returns a `BifrostResult` containing the `OperationInfo`.
///
/// # Examples
///
/// ```compile_fail
/// // Both `prep` and `build` return `BifrostResult`s containing structures that
/// // implement `BifrostOperable`.
/// let bifrost_operable = WorkSpace::to_load_space(config, &args)
///        .prep()?
///        .build()?;
///
/// // Conversely, `exec` is a consuming operation and returns a `BifrostResult`
/// // containing the `OperationInfo` (i.e. number of bytes written).
/// let operation_info = WorkSpace::to_load_space(config, &args)
///        .prep()?
///        .build()?
///        .exec()?;
/// ```
impl BifrostOperable for LoadSpace {
    /// Prepares a `LoadSpace` by ensuring that its target destination (`BifrostPath`)
    /// does not already exist. Reassigns `self.bifrost_path` if and only if the
    /// `BifrostPath` is unique.
    fn prep(&mut self) -> BifrostResult<&mut BifrostOperable> {
        if self.bifrost_path().is_some() {
            failure::bail!("error: a `BifrostPath` has already been prepared for this `LoadSpace`");
        }
        self.bifrost_path = Some(BifrostPath::new(self.home_path(), self.name())?);
        Ok(self)
    }

    /// Builds a `LoadSpace` by calling `walk` on each underlying `WorkingDir` owned
    /// by the workspace's contents. Reassigns `self.workspace.contents` if and only if
    /// each `walk` was successful.
    fn build(&mut self) -> BifrostResult<&mut BifrostOperable> {
        if let Some(contents) = self.workspace.contents.as_mut() {
            let mut walked_content: Vec<WorkingDir> = vec![];
            while let Some(entry) = contents.pop() {
                let wd = entry.walk()?;
                walked_content.push(wd);
            }
            self.workspace.contents = Some(walked_content)
        }
        Ok(self)
    }

    /// Executes a `LoadSpace`'s primary function: `load`.
    fn exec(&mut self) -> BifrostResult<OperationInfo> {
        Ok(self.load()?)
    }
}

/// A no-field struct used to signal a division of labor/responsibility.
struct WorkSpaceBuilder;

impl WorkSpaceBuilder {
    // Returns the ignore list from the manifest if it exists. Otherwise, an
    // empty vector will be returned meaning that no files will be ignored in
    // the `WorkSpace`.
    fn get_workspace_ignore_list(config: &Config) -> Vec<String> {
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
            config::values_of("contents", &args).map_or(vec![], |v| v),
        )
    }

    // Convenience function that gets the name of the workspace and the `ignore_list`
    // from a `Config`.
    fn values_from_config(config: &Config) -> (String, Vec<String>) {
        (
            WorkSpaceBuilder::get_workspace_name(&config),
            WorkSpaceBuilder::get_workspace_ignore_list(&config),
        )
    }

    // Convenience function that `strip`s trailing slashes from command line arguments,
    // `check`s if the current working directory has paths that contain these arguments,
    // and then returns absolute-path-versions of these arguments.
    // More validation may need to be done; this is a naive implementation.
    fn args_to_paths(
        config: &Config,
        args: Vec<String>,
        f: &Fn(&Config, &str) -> BifrostResult<bool>,
    ) -> Vec<PathBuf> {
        let ws_path_names = strip(&args);
        let ws_paths = check(&config, ws_path_names, f);
        return to_abs_paths(config.cwd(), ws_paths);
    }
}

// Collects `names` that can be considered valid paths for a given operation. If
// the `BifrostResult` returned by `f` cannot be unwrapped then the value an `Ok`
// result would have contained is considered to be false.
fn check(
    config: &Config,
    names: Vec<String>,
    f: &Fn(&Config, &str) -> BifrostResult<bool>,
) -> Vec<String> {
    names
        .into_iter()
        .filter(|n| f(&config, n).unwrap_or(false))
        .collect()
}

// A `name` is valid if it exists within the current Bifrost realm.
fn is_loadable(config: &Config, name: &str) -> BifrostResult<bool> {
    // Home or root paths are not loadable.
    if config.home_path().as_path() == Path::new(name) {
        return Ok(false);
    }

    // A path that already contains .bifrost or container is not loadable.
    if name.contains(".biforst") || name.contains("container") {
        return Ok(false);
    }

    // A `name` must exist within the current working directory in order for
    // it to be loadable.
    for entry in fs::read_dir(config.cwd())? {
        let entry = entry?;
        if entry.file_name() == name {
            return Ok(true);
        }
    }
    // No other cases are loadable.
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
    if name.len() > 1 && (name.ends_with("\\") || name.ends_with("/")) {
        return &name[..name.len() - 1];
    }
    name
}

// Converts `name`d strings to `PathBuf`s if and only if the name is contained
// within a `PathBuf` that resides in the current working directory.
fn to_abs_paths<P>(cwd: P, paths: Vec<String>) -> Vec<PathBuf>
where
    P: AsRef<Path>,
{
    let mut abs_paths: Vec<PathBuf> = Vec::new();
    if paths.is_empty() {
        return abs_paths;
    }

    if let Ok(read_dir) = fs::read_dir(cwd) {
        for entry in read_dir {
            let entry =
                entry.expect("BUG: `BifrostPath::check` passed but `to_abs_paths` has failed");
            for name in paths.iter() {
                if entry.file_name() == name[..] {
                    abs_paths.push(entry.path())
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default_workspace() {
        let ws = WorkSpace::default();
        let msg = "BUG: `WorkSpace::default` succeeds but `test_default_workspace` fails to";
        let home_path = dirs::home_dir().expect(&format!("{} {}", msg, "unwrap `dirs::home_dir`"));
        let cwd = std::env::current_dir()
            .expect(&format!("{} {}", msg, "unwrap `std::env::current_dir`"));

        assert_eq!(home_path, *ws.config.home_path());
        assert_eq!(cwd, *ws.config.cwd());
        assert_eq!(Mode::Normal, ws.mode);
        assert!(ws.name.is_none() && ws.contents.is_none());
    }

    #[test]
    fn test_to_abs_paths() {
        let names: Vec<String> = vec![
            String::from("file.txt"),
            String::from("file.cpp"),
            String::from("test_dir_nested"),
            String::from("test_ignore_file.txt"),
            String::from("file.c"),
            String::from("file.rs"),
            // Not valid
            String::from("random"),
            // Not valid
            String::from("not in dir"),
        ];

        const LEFT: usize = 6;
        let abs_paths = to_abs_paths("tests/test_dir", names);
        assert_eq!(LEFT, abs_paths.len());

        let abs_paths = to_abs_paths("tests/test_dir", vec![]);
        assert!(abs_paths.is_empty());
    }

    #[test]
    fn test_strip_trailing_slash() {
        let n0 = "";
        assert_eq!(n0, strip_trailing_slash(n0));

        let n1 = "/";
        assert_eq!(n1, strip_trailing_slash(n1));

        let n2 = "src/";
        assert_eq!("src", strip_trailing_slash(n2));

        let n3 = r#"src\"#;
        assert_eq!("src", strip_trailing_slash(n3));
    }

    #[test]
    fn test_strip() {
        let left: Vec<String> = vec![
            String::from(""),
            String::from("/"),
            String::from("src/"),
            String::from(r#"src\"#),
        ];

        let right = strip(&left);
        assert_eq!(left.len(), right.len());
    }

    // figure out how to run this test
    fn _test_is_loadable() -> BifrostResult<()> {
        let current_dir = std::env::current_dir()?;
        let test_path = Path::new("tests").join("test_dir");
        std::env::set_current_dir(test_path)?;
        let config = Config::default();
        let mut entries: Vec<String> = vec![];
        for e in fs::read_dir(config.cwd())? {
            let e = e?;
            let fname = e.file_name();
            let s = fname
                .into_string()
                .expect("error: test_is_loadable failed to convert `file_name` `into_string`");
            entries.push(s);
        }

        for e in entries {
            assert!(is_loadable(&config, &e)?);
        }
        std::env::set_current_dir(current_dir)?;
        Ok(())
    }
}
