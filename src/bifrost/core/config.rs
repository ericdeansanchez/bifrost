//! Primary configuration structure and utilities for Bifrost realms.
use serde_derive::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

use crate::core::hofund;
use crate::util::BifrostResult;
use crate::ArgMatches;

use dirs;
use failure;

// [TODO] More thought should be put forward concerning `Config` and `ArgMatches`
// At some point, `Config` should be the only thing needed or at least it should
// look something like this:
// [config] + [manifest] + [arg_matches] = [the best of all worlds]

/// Primary configuration structure for Bifrost realms.
///
/// A `Config` carries the necessary information a Bifrost realm needs to
/// carry out `bifrost::ops`. It is composed of the following:
/// * `home_path` - a `PathBuf` to a user's home directory.
/// * `cwd` - a `PathBuf` to a user's current working directory.
/// * `manifest` - a `BifrostManifest` constructed from a `Bifrost.toml` file and `clap::ArgMatches`
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// Absolute path to a user's home directory.
    home_path: PathBuf,
    /// Absolute path to a user's current working directory.
    cwd: PathBuf,
    /// The [`BifrostManifest`] contains workspace configuration information.
    /// This structure can be constructed either from:
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
/// This function will also fail if the current working directory is found to be
/// the home or root directory. In this case, `default` will panic if it cannot
/// write to `io::stderr`.
///
/// If any of these calls fails to provide an appropriate value, then the program
/// should terminate.
impl Default for Config {
    fn default() -> Self {
        let home_path =
            dirs::home_dir().expect("error: `core::Config::default` could not configure home path");
        let cwd = env::current_dir()
            .expect("error: `core::Config::default` could not configure cwd path");

        // Do not let users construct a Bifrost realm from within _THE_ Bifrost
        // directory or the Bifrost container directory.
        let bifrost_path = home_path.as_path().join(".bifrost");
        let container_path = home_path.as_path().join(".bifrost").join("container");

        if cwd == home_path
            || cwd == PathBuf::from("/")
            || cwd == bifrost_path
            || cwd == container_path
        {
            eprintln!("error: cannot configure root directory as a Bifrost realm");
            process::exit(1);
        }

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
            _ => None,
        }
    }

    pub fn manifest_mut(&mut self) -> Option<&mut BifrostManifest> {
        self.manifest.as_mut()
    }
}

/// Primary structure to serialize and deserialize Bifrost manifest data.
#[derive(Debug, Deserialize, Serialize)]
pub struct BifrostManifest {
    workspace: Option<WorkSpaceConfig>,
    container: Option<ContainerConfig>,
    command: Option<CommandConfig>,
}

/// Constructs a default `BifrostManifest` from a raw string literal.
/// This string is then toml-fied via `toml::from_str`.
impl Default for BifrostManifest {
    fn default() -> Self {
        let default_manifest = r#"[workspace]
name = "name of current workspace"
ignore = ["target", ".git", ".gitignore"]
[container]
name = "docker"
[command]
cmds = ["default-arg"]
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
        match BifrostManifest::from_manifest(&config) {
            Ok(manifest) => {
                if args.args.is_empty() {
                    return manifest;
                } else {
                    return manifest.combine_with(&args);
                }
            }
            Err(_) => BifrostManifest::default(),
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

        let s = match hofund::read(&cwd.join("Bifrost.toml")) {
            Ok(s) => s,
            Err(e) => {
                io::stderr().write_fmt(format_args!(
                    "error: could not read Bifrost.toml due to `{}`\n",
                    e
                ))?;
                process::exit(1);
            }
        };

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

        self.command = match self.command {
            Some(c) => Some(c.combine_with(&args)),
            None => self.command,
        };

        self
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

    /// Gets a reference to the manifest's `WorkSpaceConfig`.
    pub fn get_workspace_config(&self) -> Option<&WorkSpaceConfig> {
        self.workspace.as_ref()
    }

    /// Gets a reference to the manifest's `BinaryConfig`.
    pub fn get_command_config(&self) -> Option<&CommandConfig> {
        self.command.as_ref()
    }

    pub fn take_command_config(&mut self) -> Option<CommandConfig> {
        self.command.take()
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
/// this function panics.
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
pub struct CommandConfig {
    cmds: Option<Vec<String>>,
}

impl Clone for CommandConfig {
    fn clone(&self) -> Self {
        CommandConfig {
            cmds: self.cmds.as_ref().map(|a| a.clone()),
        }
    }
}

impl CommandConfig {
    fn combine_with(self, arg_matches: &ArgMatches) -> Self {
        if arg_matches.args.is_empty() {
            return self;
        }

        match values_of("commands", &arg_matches) {
            Some(mut arg_cmds) => {
                if let Some(cmds) = self.cmds {
                    for c in cmds {
                        arg_cmds.push(c);
                    }
                    return CommandConfig {
                        cmds: Some(arg_cmds),
                    };
                } else {
                    return CommandConfig {
                        cmds: Some(arg_cmds),
                    };
                }
            }
            None => self,
        }
    }
}

impl Default for CommandConfig {
    fn default() -> Self {
        CommandConfig {
            cmds: Some(vec![String::from("ls")]),
        }
    }
}

impl CommandConfig {
    pub fn get_cmds(&self) -> Option<&Vec<String>> {
        self.cmds.as_ref()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default_config() {
        // Ensure the default constructor does not panic.
        let _ = Config::default();
    }

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

        let ws_config = manifest.get_workspace_config().expect(&format!(
            "{} {}",
            msg, "`BifrostManifest.workspace` cannot be unwrapped"
        ));

        let left = "workspace name";
        assert_eq!(
            left,
            ws_config.name.as_ref().expect(&format!(
                "{} {}",
                msg, "`WorkSpaceConfig.name` cannot be unwrapped"
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

        let left = vec![String::from("test")];
        assert_eq!(
            Some(left.as_ref()),
            manifest
                .get_command_config()
                .expect(&format!(
                    "{} {}",
                    msg, "`CommandConfig` cannot be unwrapped"
                ))
                .get_cmds()
        );
    }
}
