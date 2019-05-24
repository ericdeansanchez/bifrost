//! Primary configuration structure and utilities for Bifrost realms.
use serde_derive::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::core::hofund;
use crate::util::BifrostResult;
use crate::ArgMatches;

use dirs;
use failure;

/// Primary configuration structure for Bifrost realms.
#[derive(Debug)]
pub struct Config {
    /// Absolute path to a user's home directory.
    home_path: PathBuf,
    /// Absolute path to a user's current working directory.
    cwd: PathBuf,
    /// The Bifrost manifest containing workspace configuration information.
    /// This structure can be constructed either `from_only` `ArgMatches` or from
    /// another Bifrost.toml manifest.
    manifest: Option<BifrostManifest>,
}

/// Constructs a default `Config`.
///
/// # Panics
///
/// This function panics if either of the following fail:
///
/// * `dirs::home_dir()`
/// * `env::current_dir()`
///
/// If either of these calls fail to provide an appropriate value then the program
/// should terminate.
impl Default for Config {
    fn default() -> Self {
        let home_path =
            dirs::home_dir().expect("error: `core::Config::default` could not configure home path");
        let cwd = env::current_dir()
            .expect("error: `core::Config::default` could not configure cwd path");
        Config {
            home_path,
            cwd,
            manifest: None,
        }
    }
}

impl Config {
    /// Configures a `Config` with a new manifest.
    ///
    /// Consumes the current `Config` to promote a new state in which an updated
    /// manifest is constructed from either:
    ///
    /// * `BifrostManifest::from_only`, which constructs a manifest from _only_
    ///   arguments passed via the command line.
    ///
    /// or
    ///
    /// * `BifrostManifest::combine_with`, which constructs a manifest from the
    /// combination of values from an existing manifest and any command line
    /// arguments that may have been passed.
    pub fn config_manifest(self, args: &ArgMatches) -> Self {
        Config {
            home_path: self.home_path,
            cwd: self.cwd,
            manifest: match self.manifest {
                None => BifrostManifest::from_only(&args),
                Some(m) => m.combine_with(&args),
            },
        }
    }

    /// Returns a reference to the underlying home directory `PathBuf`.
    pub fn home_path(&self) -> &PathBuf {
        &self.home_path
    }

    /// Returns a reference to the underlying current working directory `PathBuf`.
    pub fn cwd(&self) -> &PathBuf {
        &self.cwd
    }

    /// Returns a reference to the underlying `BifrostManifest`.
    pub fn manifest(&self) -> Option<&BifrostManifest> {
        match &self.manifest {
            Some(m) => Some(&m),
            None => None,
        }
    }
}

/// Primary structure to serialize and deserialize Bifrost manifest data.
#[derive(Debug, Deserialize, Serialize)]
pub struct BifrostManifest {
    project: Option<ProjectConfig>,
    container: Option<ContainerConfig>,
    workspace: Option<WorkSpaceConfig>,
    command: Option<CommandConfig>,
}

/// Constructs the default `BifrostManifest`.
impl Default for BifrostManifest {
    fn default() -> Self {
        let default_manifest = r#"[project]
name = "project name"

[container]
name = "docker"

[workspace]
name = "name of current workspace"
ignore = ["target", ".git", ".gitignore"]

[command]
cmd = ["command string"]
"#;
        match toml::from_str(&default_manifest) {
            Ok(valid_toml) => valid_toml,
            Err(e) => panic!(
                "BUG: {} attempting to construct \
                 `BifrostManifest::default` with invalid toml",
                e
            ),
        }
    }
}

impl BifrostManifest {
    /// Constructs a `BifrostManifest` from an existing Bifrost.toml manifest file.
    ///
    /// # Errors
    ///
    /// If no Bifrost.toml manifest is found in the current working directory,
    /// then the `Config` cannot be constructed from an existing manifest.
    ///
    /// If the Bifrost.toml manifest could not be serialized from a string, then
    /// this also results in an error.
    pub fn from_manifest(config: &Config) -> BifrostResult<Self> {
        let cwd = config.cwd();

        if fs::metadata(&cwd.join("Bifrost.toml")).is_err() {
            failure::bail!(
                "error: could not find `Bifrost.toml` with the current working directory"
            )
        }

        let s = hofund::read(&cwd.join("Bifrost.toml"))?;

        match toml::from_str(&s) {
            Ok(b) => Ok(b),
            Err(e) => failure::bail!(
                "error: could not serialize `Bifrost.toml` from string due to {}",
                e
            ),
        }
    }

    /// Constructs a _possibly_ new `BifrostManifest`.
    ///
    /// This method consumes the current manifest and replaces its fields with
    /// fields passed as `clap::ArgMatches` (if they exist). Only fields that
    /// were explicitly passed will be replaced; otherwise, the `BifrostManifest`'s
    /// fields will contain its previously owned values.
    pub fn combine_with(mut self, args: &ArgMatches) -> Option<Self> {
        self.project = match value_of("project", &args) {
            None => self.project,
            Some(p) => Some(ProjectConfig { name: Some(p) }),
        };

        self.container = match value_of("container", &args) {
            None => self.container,
            Some(c) => Some(ContainerConfig { name: Some(c) }),
        };

        self.workspace = match value_of("workspace", &args) {
            None => self.workspace,
            Some(ws) => {
                let ignore = match values_of("ignore", &args) {
                    None => self.workspace.unwrap().ignore,
                    Some(ig) => Some(ig),
                };
                Some(WorkSpaceConfig {
                    name: Some(ws),
                    ignore,
                })
            }
        };

        self.command = match values_of("command", &args) {
            None => self.command,
            Some(c) => Some(CommandConfig { cmd: Some(c) }),
        };

        Some(self)
    }

    /// Constructs a `BifrostManifest` from _only_ command line arguments.
    ///
    /// This function constructs a `BifrostManifest` from each value of
    /// command line argument that is present in the `clap::ArgMatches`.
    ///
    /// Each field that is not present is constructed with default arguments.
    pub fn from_only(args: &ArgMatches) -> Option<Self> {
        Some(BifrostManifest {
            project: match value_of("project", &args) {
                None => Some(ProjectConfig::new("project name")),
                Some(p) => Some(ProjectConfig::new(&p)),
            },
            container: match value_of("container", &args) {
                None => Some(ContainerConfig::new("docker")),
                Some(c) => Some(ContainerConfig { name: Some(c) }),
            },
            workspace: match value_of("workspace", &args) {
                None => Some(WorkSpaceConfig::new(
                    "name of workspace",
                    vec![
                        String::from("target"),
                        String::from(".git"),
                        String::from(".gitignore"),
                    ],
                )),
                Some(ws) => {
                    let ignore = match values_of("ignore", &args) {
                        None => Some(vec![
                            String::from("target"),
                            String::from(".git"),
                            String::from(".gitignore"),
                        ]),
                        Some(ig) => Some(ig),
                    };
                    Some(WorkSpaceConfig {
                        name: Some(ws),
                        ignore,
                    })
                }
            },
            command: match values_of("command", &args) {
                None => Some(CommandConfig::new(vec![String::from("command string(s)")])),
                Some(c) => Some(CommandConfig { cmd: Some(c) }),
            },
        })
    }

    /// Returns a `BifrostResult`.
    ///
    /// If this value is `Ok`, then this result contains the string representation
    /// of the TOML-serialized `BifrostManifest`.
    pub fn to_str(&self) -> BifrostResult<String> {
        match toml::to_string(&self) {
            Ok(s) => Ok(String::from(s)),
            Err(e) => failure::bail!(format!(
                "error: could not convert `BifrostManifest` to string due to `{}`",
                e
            )),
        }
    }
}

fn value_of(arg: &str, from_args: &ArgMatches) -> Option<String> {
    if from_args.is_present(arg) {
        return from_args
            .value_of(arg)
            .map_or(None, |s| Some(String::from(s)));
    }
    None
}

fn values_of(arg: &str, from_args: &ArgMatches) -> Option<Vec<String>> {
    if from_args.is_present(arg) {
        let values: Vec<&str> = from_args
            .values_of(arg)
            .expect(&format!("error: could not parse `values_of` `{}`", arg))
            .collect();
        return Some(values.iter().map(|s| String::from(*s)).collect());
    }
    None
}

#[derive(Debug, Deserialize, Serialize)]
struct ContainerConfig {
    name: Option<String>,
}

impl ContainerConfig {
    fn new(name: &str) -> Self {
        ContainerConfig {
            name: Some(String::from(name)),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct ProjectConfig {
    name: Option<String>,
}

impl ProjectConfig {
    fn new(name: &str) -> Self {
        ProjectConfig {
            name: Some(String::from(name)),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct WorkSpaceConfig {
    name: Option<String>,
    ignore: Option<Vec<String>>,
}

impl WorkSpaceConfig {
    fn new(name: &str, ignore: Vec<String>) -> Self {
        WorkSpaceConfig {
            name: Some(String::from(name)),
            ignore: Some(ignore),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct CommandConfig {
    cmd: Option<Vec<String>>,
}

impl CommandConfig {
    fn new(cmd: Vec<String>) -> Self {
        CommandConfig { cmd: Some(cmd) }
    }
}

#[derive(Debug)]
pub enum CommandOptions {
    Load(LoadOptions),
    Show(ShowOptions),
    Run(RunOptions),
    Unload(UnloadOptions),
}

#[derive(Debug)]
pub struct LoadOptions {
    src: PathBuf,
    dst: BifrostPath,
}

#[derive(Debug)]
pub struct ShowOptions {
    path: BifrostPath,
}

#[derive(Debug)]
pub struct RunOptions {
    path: BifrostPath,
}

#[derive(Debug)]
pub struct UnloadOptions {
    path: BifrostPath,
}

#[derive(Debug)]
pub struct BifrostPath {
    path: PathBuf,
}

impl BifrostPath {
    fn _new<P: AsRef<Path>>(path: P) -> BifrostResult<Self> {
        let black_list = [
            ".bifrost",
            "bifrost",
            ".config",
            "config",
            "container",
            "dockerfile",
            "Docker",
            "Dockerfile",
            "test",
        ];

        match path.as_ref().file_name().and_then(|p| p.to_str()) {
            Some(s) => {
                if black_list.contains(&s) || s.starts_with(".") {
                    failure::bail!(
                        "error: cannot create proposed path {} because it
                    conflicts with one of the following: {:?}",
                        s,
                        black_list
                    )
                } else {
                    return Ok(BifrostPath {
                        path: path.as_ref().to_path_buf(),
                    });
                }
            }
            None => failure::bail!("error: cannot verify proposed path, path may have been empty"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default_manifest() {
        // This test is to ensure that `BifrostManifest::default` does
        // not panic.
        let _dm = BifrostManifest::default();
    }

    #[test]
    fn test_bifrost_manifest_from_manifest() {
        pub fn from_manifest(path: &Path) -> BifrostResult<BifrostManifest> {
            if fs::metadata(&path.join("TestBifrost.toml")).is_err() {
                failure::bail!(
                    "error: could not find `TestBifrost.toml` with the current working directory"
                )
            }
            let s = hofund::read(&path.join("TestBifrost.toml"))?;
            match toml::from_str(&s) {
                Ok(b) => Ok(b),
                Err(e) => failure::bail!(
                    "error: could not serialize `Bifrost.toml` from string due to {}",
                    e
                ),
            }
        }
        assert!(from_manifest(Path::new("tests/test_user/test_app_dir")).is_ok());
    }
}
