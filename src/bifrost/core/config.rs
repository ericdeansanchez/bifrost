use std::env;
use std::path::{Path, PathBuf};

use crate::util::BifrostResult;

use dirs;
use failure;

#[derive(Debug)]
pub struct Config {
    home_path: PathBuf,
    cwd: PathBuf,
    opts: Option<CommandOptions>,
}

impl Default for Config {
    fn default() -> Self {
        let home_path = dirs::home_dir().expect("error: could not configure home path");
        let cwd = env::current_dir().expect("error: `Config::default` could not configure cwd");
        Config {
            home_path,
            cwd,
            opts: None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bifrost_path() {
        assert!(BifrostPath::new(Path::new("configuration")).is_ok());
        assert!(BifrostPath::new(Path::new("config")).is_err());
        assert!(BifrostPath::new(Path::new("container")).is_err());
        assert!(BifrostPath::new(Path::new("Dockerfile")).is_err());
    }
}
