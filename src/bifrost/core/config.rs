//! Primary configuration structure and utilities for Bifrost realms.
use serde_derive::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

use crate::core::hofund;
use crate::util::BifrostResult;
use crate::ArgMatches;

use dirs;
use failure;

/// Primary configuration structure for Bifrost realms.
///
/// A `Config` carries the necessary information a Bifrost realm needs to
/// carry out `bifrost::ops`. It is composed of the following:
/// * `home_path` - a `PathBuf` to a user's home directory.
/// * `cwd` - a `PathBuf` to a user's current working directory.
/// * `manifest` - a `BifrostManifest` constructed from a `Bifrost.toml` file and `clap::ArgMatches`
#[derive(Debug)]
pub struct Config {
    /// Absolute path to a user's home directory.
    home_path: PathBuf,
    /// Absolute path to a user's current working directory.
    cwd: PathBuf,
    /// The Bifrost manifest contains workspace configuration information.
    /// This structure can be constructed
    /// * `from_only` `ArgMatches` or
    /// * another Bifrost.toml manifest or
    /// * the combination of `ArgMatches` and `BifrostManifest`
    manifest: Option<BifrostManifest>,
}

/// Constructs a default `Config`.
///
/// # Panics
///
/// The `default` function panics if either of the following fail:
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
    /// Initializes this `Config`s manifest. This method differs from `Config::config_manifest`
    /// in that it expects `self.manifest` to be `None`. In this case, this method
    /// returns a new `BifrostManifest` which is either a default or a default
    /// combined with any valid command line arguments that were passed. This method
    /// should only be called in `bifrost_init::init`.
    pub fn init_manifest(self, args: &ArgMatches) -> Self {
        Config {
            home_path: self.home_path.clone(),
            cwd: self.cwd.clone(),
            manifest: match self.manifest {
                None => Some(BifrostManifest::new(&args)),
                Some(m) => {
                    eprintln!(
                        "BUG: reinitializing manifest {:#?} `manifest` \
                         was Some prior to intialization",
                        m
                    );
                    process::exit(1);
                }
            },
        }
    }

    /// Configures a `Config` with a new manifest. Consumes the current `Config`
    /// to promote a new state in which an updated manifest is constructed from either:
    ///
    /// * `BifrostManifest::manifest_or_bust` or
    /// * [`BifrostManifest::combine_with`](struct.BifrostManifest.html#method.combine_with),
    /// which constructs a manifest from the combination of values from an existing
    /// manifest and any command line arguments that may have been passed.
    ///
    /// This method differs from [`Config::init_manifest`](struct.Config.html#method.init_manifest)
    /// in that it will error and exit if the current working directory has not
    /// been initialized as a Bifrost realm.
    ///
    /// Consequently, this method should only be called _after_ Bifrost-realm-initialization.
    pub fn config_manifest(self, args: &ArgMatches) -> Self {
        Config {
            home_path: self.home_path.clone(),
            cwd: self.cwd.clone(),
            manifest: match self.manifest {
                None => Some(BifrostManifest::manifest_or_bust(self, &args)),
                Some(m) => Some(m.combine_with(&args)),
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

/// Constructs a default `BifrostManifest` from a raw string literal.
/// This string is then toml-fied via `toml::from_str`.
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
    /// Constructs a new manifest that is the result of combining a default
    /// manifest with `clap::ArgMatches` if they exist. Otherwise, the default
    /// manifest is returned.
    pub fn new(args: &ArgMatches) -> Self {
        if args.args.is_empty() {
            return BifrostManifest::default();
        }
        return BifrostManifest::default().combine_with(&args);
    }

    /// Returns a valid manifest or prompts user with an error message and exits.
    ///
    /// First, an attempt is made to read the manifest within the current working
    /// directory.
    ///
    /// If this operation succeeds, then the manifest is combined with
    /// command line arguments (if they are present).
    ///
    /// Otherwise, this method `bail`s returning the error message to be displayed
    /// to the user before exiting.
    fn manifest_or_bust(config: Config, args: &ArgMatches) -> Self {
        fn try_from_manifest(config: Config, args: &ArgMatches) -> BifrostResult<BifrostManifest> {
            match BifrostManifest::from_manifest(&config) {
                Ok(manifest) => {
                    if args.args.is_empty() {
                        return Ok(manifest);
                    } else {
                        return Ok(manifest.combine_with(&args));
                    }
                }
                Err(e) => failure::bail!("error: not a valid Bifrost realm\n{}", e),
            }
        }

        match try_from_manifest(config, &args) {
            Ok(manifest) => manifest,
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
    }

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
                "--could not find `Bifrost.toml`
       within the current working directory.

note: a Bifrost realm must be intialized before use.

try:

    bifrost init

"
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
    pub fn combine_with(mut self, args: &ArgMatches) -> Self {
        self.project = match value_of("project", &args) {
            None => self.project,
            Some(p) => Some(ProjectConfig { name: Some(p) }),
        };

        self.container = match value_of("container", &args) {
            None => self.container,
            Some(c) => Some(ContainerConfig { name: Some(c) }),
        };

        self.workspace = match value_of("workspace", &args) {
            None => match values_of("ignore", &args) {
                None => self.workspace,
                Some(ig) => {
                    let mut list: Vec<String> = vec![];
                    for i in ig {
                        list.push(i);
                    }
                    Some(WorkSpaceConfig::new("name of workspace", list))
                }
            },
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

        self
    }

    /// Constructs a `BifrostManifest` from _only_ command line arguments.
    ///
    /// This function constructs a `BifrostManifest` from each value of
    /// command line argument that is present in the `clap::ArgMatches`.
    ///
    /// Each field that is not present is constructed with default arguments.
    pub fn from_only(args: &ArgMatches) -> Self {
        BifrostManifest {
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
        }
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

    /// Returns an optional reference to this manifest's `WorkSpaceConfig`.
    pub fn workspace_config(&self) -> Option<&WorkSpaceConfig> {
        self.workspace.as_ref()
    }
}

/// Light wrapper around `clap`s `value_of` method.
pub fn value_of(arg: &str, from_args: &ArgMatches) -> Option<String> {
    if from_args.is_present(arg) {
        return from_args
            .value_of(arg)
            .map_or(None, |s| Some(String::from(s)));
    }
    None
}

/// Light wrapper around `clap`s `values_of` method.
///
/// # Panics
///
/// In the case where the given `arg` `is_present` but its values cannot be unwrapped
/// this function pan
pub fn values_of(arg: &str, from_args: &ArgMatches) -> Option<Vec<String>> {
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
pub struct WorkSpaceConfig {
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

    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|n| n.as_ref())
    }

    pub fn ignore(&self) -> Option<&Vec<String>> {
        self.ignore.as_ref().map(|i| i.as_ref())
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
    fn test_from_manifest() {
        // This reads the the Bifrost.toml manifest from the testing directory.
        // This is meant to succeed and all values are within the toml file are
        // meant to be present. If this test fails, then it could be that the
        // toml file was corrupted or modified.
        let mut config = Config::default();
        config.cwd = PathBuf::from("tests")
            .join("test_user")
            .join("test_app_dir");

        let manifest = BifrostManifest::from_manifest(&config);
        assert!(manifest.is_ok());

        // The manifest exists and it is safe to unwrap.
        let msg = "BUG: manifest `is_ok` but";
        let manifest = manifest.unwrap();
        let left = String::from("project name");
        assert_eq!(
            left,
            *manifest
                .project
                .as_ref()
                .expect(&format!(
                    "{} {}",
                    msg, "`ProjectConfig` cannot be unwrapped"
                ))
                .name
                .as_ref()
                .expect(&format!(
                    "{} {}",
                    msg, "`ProjectConfig::name` cannot be unwrapped"
                ))
        );

        let left = String::from("docker");
        assert_eq!(
            left,
            *manifest
                .container
                .as_ref()
                .expect(&format!(
                    "{} {}",
                    msg, "`ContainerConfig` cannot be unwrapped"
                ))
                .name
                .as_ref()
                .expect(&format!(
                    "{} {}",
                    msg, "`ContainerConfig::name` cannot be unwrapped"
                ))
        );

        let ws_config = manifest.workspace_config().expect(&format!(
            "{} {}",
            msg, "`BifrostManifest::workspace_config` cannot be unwrapped"
        ));

        let left = "name of workspace";
        assert_eq!(
            left,
            ws_config.name.as_ref().expect(&format!(
                "{} {}",
                msg, "`WorkSpaceConfig::name` cannot be unwrapped"
            ))
        );

        let left = vec![
            String::from("target"),
            String::from(".git"),
            String::from(".gitignore"),
        ];

        assert_eq!(
            &left,
            ws_config.ignore.as_ref().expect(&format!(
                "{} {}",
                msg, "`WorkSpaceConfig::ignore` cannot be unwrapped"
            ))
        );

        let left = vec![String::from("command string(s)")];
        assert_eq!(
            left,
            manifest
                .command
                .expect(&format!(
                    "{} {}",
                    msg, "`CommandConfig` cannot be unwrapped"
                ))
                .cmd
                .expect(&format!(
                    "{} {}",
                    msg, "`CommandConfig::cmd` cannot be unwrapped"
                ))
        );
    }
}
