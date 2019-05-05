//! Basic configuration utilities.
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;

const AUTO: &str = "AUTO";
const IGNORE_ON_LOAD: &str = "IGNORE_ON_LOAD";
const IGNORE_ON_SHOW: &str = "IGNORE_ON_SHOW";
const MODIFIED: &str = "MODIFIED";

/// Primary configuration object. When the .config file is
/// parsed `Config` takes ownership of the path for future
/// `write`s.
#[derive(Debug)]
pub struct Config {
    /// If `--auto` is passed or a `true` value is parsed from the .config file,
    /// then reloading files into container manually via `--modified` is not
    /// required. Modified files will be detected and loaded upon calls to `run`.
    pub auto: bool,

    /// If `true`, `$ bifrost load .` will ignore all contents of the
    /// `WorkSpace.ignore_list` when loading the current work space into the
    /// container.
    pub ignore_on_load: bool,

    /// If `true`, `$ bifrost show` will ignore all contents of the
    /// `WorkSpace.ignore_list` when displaying the currently loaded work space.
    pub ignore_on_show: bool,

    /// If the `--modified` flag is passed then only modified files will be
    /// loaded.
    pub modified: bool,

    /// The path to the configuration file.
    pub path: Option<PathBuf>,
}

impl Config {
    /// Initializes `Config` from .config file. The .config file is parsed by the
    /// associated method, `Config::parse`, which populates the `config_map`.
    /// There is some very basic validation going on here with `Config::validate`.
    ///
    /// Moreover, there are better configuration methods (i.e. other crates),
    /// however, for the moment we are trying to keep dependencies to a minimum.
    pub fn init(path: Option<PathBuf>) -> Config {
        let mut config_map: HashMap<String, bool> = HashMap::new();

        Config::parse(&path, &mut config_map);

        let auto = Config::validate(&AUTO, &config_map);
        let ignore_on_load = Config::validate(&IGNORE_ON_LOAD, &config_map);
        let ignore_on_show = Config::validate(&IGNORE_ON_SHOW, &config_map);
        let modified = Config::validate(&MODIFIED, &config_map);

        Config {
            auto,
            ignore_on_load,
            ignore_on_show,
            modified,
            path,
        }
    }

    /// This is a light wrapper around `fs::write`. If the path exist
    /// then the contents are written to the file specified as a byte string.
    pub fn write(&self, contents: &str) -> io::Result<()> {
        if let Some(path) = &self.path {
            fs::write(path, contents.as_bytes())?;
        }

        Ok(())
    }

    /// Lightly validates config_map values. This mostly exists to limit the
    /// size of `Config::init`.
    fn validate(key: &str, config_map: &HashMap<String, bool>) -> bool {
        config_map.get::<str>(key).map_or(false, |value| *value)
    }

    /// Very basic parsing function based on a very basic configuration file.
    /// Each line in the .config file has the form `key=value`.
    fn parse(path: &Option<PathBuf>, config_map: &mut HashMap<String, bool>) {
        if let Some(path) = path {
            let env: Vec<u8> = fs::read(path).unwrap();
            let env: Cow<str> = String::from_utf8_lossy(&env);
            let env: Vec<&str> = env.split_terminator("\n").collect();

            for v in env {
                let pair: Vec<&str> = v.split_terminator("=").collect();

                config_map.insert(
                    String::from(pair[0]),
                    FromStr::from_str(pair[1]).unwrap_or(false),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dir;

    #[test]
    fn test_init_config() {
        let path = dir::path_builder(
            Some(PathBuf::from("tests/test_user")),
            "test_app_dir/.test_config",
        );

        // The .test_config file begins with each variable set to false;
        // ensure `Config` is initialized accordingly.
        let c = Config::init(path);
        assert!(!c.auto && !c.ignore_on_show && !c.ignore_on_load && !c.modified);
    }

    #[test]
    fn test_write_config() -> io::Result<()> {
        let path = dir::path_builder(
            Some(PathBuf::from("tests/test_user")),
            "test_app_dir/.test_config",
        );

        let c = Config::init(path);
        assert!(!c.auto && !c.ignore_on_show && !c.ignore_on_load && !c.modified);

        let all_true: &str = "AUTO=true\n\
                              IGNORE_ON_SHOW=true\n\
                              IGNORE_ON_LOAD=true\n\
                              MODIFIED=true";

        let _result = c.write(all_true);

        let path = dir::path_builder(
            Some(PathBuf::from("tests/test_user")),
            "test_app_dir/.test_config",
        );

        let c = Config::init(path);
        assert!(c.auto && c.ignore_on_show && c.ignore_on_load && c.modified);

        let all_false: &str = "AUTO=false\n\
                               IGNORE_ON_SHOW=false\n\
                               IGNORE_ON_LOAD=false\n\
                               MODIFIED=false";

        let result = c.write(all_false);

        let path = dir::path_builder(
            Some(PathBuf::from("tests/test_user")),
            "test_app_dir/.test_config",
        );

        // .test_config must return to its initial state.
        let c = Config::init(path);
        assert!(!c.auto && !c.ignore_on_show && !c.ignore_on_load && !c.modified);

        result
    }

    #[test]
    fn test_config_parse() {
        let mut config_map: HashMap<String, bool> = HashMap::new();
        let path = dir::path_builder(
            Some(PathBuf::from("tests/test_user")),
            "test_app_dir/.test_config",
        );

        // The .test_config file begins with each variable
        // set to `false`.
        let test_map = vec![
            ("AUTO", false),
            ("IGNORE_ON_SHOW", false),
            ("IGNORE_ON_LOAD", false),
            ("MODIFIED", false),
        ];

        // Parse the config file and ensure that the parsed key-value
        // pairs are consistent with what we expect.
        Config::parse(&path, &mut config_map);
        for (k, v) in test_map {
            assert_eq!(v, *config_map.get::<str>(&k).unwrap());
        }
    }
}
