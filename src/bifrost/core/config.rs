use std::collections::HashMap;
use std::env;
use serde_derive::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::util::BifrostResult;

use dirs;
use failure;

#[derive(Debug)]
pub struct Config {
    home_path: PathBuf,
    cwd: PathBuf,
    manifest: Option<BifrostManifest>,
    opts: Option<CommandOptions>,
    cli_args: Option<Vec<String>>,
    env_vars: Option<HashMap<String, String>>,
}

impl Default for Config {
    fn default() -> Self {
        let home_path = dirs::home_dir().expect("error: could not configure home path");
        let cwd = env::current_dir().expect("error: `Config::default` could not configure cwd");
        Config {
            home_path,
            cwd,
            manifest: None,
            opts: None,
            cli_args: None,
            env_vars: None,
        }
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
    fn new<P: AsRef<Path>>(path: P) -> BifrostResult<Self> {
        let black_list = [
            ".bifrost",
            "bifrost",
            ".config",
            "config",
            "container",
            "docker",
            "Docker",
            "Dockerfile",
            "test",
        ];

        match path.as_ref().file_name().and_then(|p| p.to_str()) {
            Some(s) => {
                if black_list.contains(&s) {
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
            None => failure::bail!(
                "error: cannot verify proposed path, path may have been empty"
            ),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct BifrostGlobalManifest {
    home_dir: Option<String>,
    default: Option<BifrostManifest>,
}

#[derive(Debug, Deserialize, Serialize)]
struct BifrostManifest {
    container: Option<ContainerConfig>,
    project: Option<ProjectConfig>,
    workspace: Option<WorkSpaceConfig>,
    command: Option<CommandConfig>,
}

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
        let default_manifest: BifrostManifest = toml::from_str(&default_manifest).unwrap();
        default_manifest
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct ContainerConfig {
    name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ProjectConfig {
    name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct WorkSpaceConfig {
    name: Option<String>,
    ignore: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CommandConfig {
    cmd: Option<Vec<String>>,
}