use std::fs;
use std::path::{Path, PathBuf};

use crate::core::config::{values_of, Config};
use crate::core::working_dir::WorkingDir;
use crate::util::BifrostResult;
use crate::ArgMatches;

#[derive(Debug)]
pub struct WorkSpace {
    name: Option<String>,
    mode: Mode,
    config: Config,
    contents: Option<Vec<WorkingDir>>,
    bifrost_path: Option<BifrostPath>,
}

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
    pub fn init(config: Config, args: &ArgMatches) -> Self {
        let ws_name = WorkSpace::get_workspace_name(&config);
        let ws_mode = flag(&args);

        let ws_ignore_list = WorkSpace::build_ignore_list(&config);
        let ws_list: Vec<&str> = ws_ignore_list.iter().map(|s| s.as_ref()).collect();

        let ws_args = values_of("contents", &args).map_or(vec![], |v| v);
        let ws_path_names = strip(&ws_args);
        let ws_paths = check(config.cwd(), ws_path_names);
        let ws_abs_paths = to_abs_path(config.cwd(), ws_paths);

        let mut contents: Vec<WorkingDir> = Vec::new();
        for path in ws_abs_paths {
            contents.push(WorkingDir::new(path).ignore(&ws_list));
        }

        WorkSpace {
            name: Some(ws_name),
            mode: ws_mode,
            config,
            contents: Some(contents),
            bifrost_path: None,
        }
    }

    fn build_ignore_list(config: &Config) -> Vec<String> {
        match config
            .manifest()
            .and_then(|m| m.workspace_config().and_then(|ws| ws.ignore()))
        {
            Some(list) => list.to_owned(),
            None => Vec::new(),
        }
    }

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
}

#[derive(Copy, Clone, Debug)]
enum Mode {
    Auto,
    Modified,
    Normal,
}

fn check<P>(cwd: P, names: Vec<String>) -> Vec<String>
where
    P: AsRef<Path>,
{
    names
        .into_iter()
        .filter(|n| is_valid(&cwd, n).unwrap_or(false))
        .collect()
}

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

fn strip(names: &Vec<String>) -> Vec<String> {
    let mut names_without_trailing_slashes: Vec<String> = Vec::new();
    for n in names {
        names_without_trailing_slashes.push(String::from(strip_trailing_slash(n)));
    }
    names_without_trailing_slashes
}

fn strip_trailing_slash(name: &str) -> &str {
    if name.len() > 1 && name.ends_with("\\") || name.ends_with("/") {
        return &name[..name.len() - 1];
    }
    name
}

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

fn flag(args: &ArgMatches) -> Mode {
    if args.is_present("auto") {
        return Mode::Auto;
    } else if args.is_present("modified") {
        return Mode::Modified;
    }
    Mode::Normal
}

#[derive(Debug)]
pub struct BifrostPath {
    path: PathBuf,
}

impl BifrostPath {
    fn new<P: AsRef<Path>>(path: P, config: &Config) -> Self {
        if let Ok(bfp) = BifrostPath::check(path) {
            let home_path = config.home_path();
            let bifrost_dir = home_path.join(".bifrost");
            let bifrost_container = bifrost_dir.join("container");
            let path = bifrost_container.join(bfp.path);
            return BifrostPath { path };
        }
        unimplemented!()
    }

    fn check<P: AsRef<Path>>(path: P) -> BifrostResult<Self> {
               let black_list = [
            ".bifrost",
            "bifrost",
            ".config",
            "config",
            "container",
            "dockerfile",
            "Dockerfile",
            "test",
            "tmp",
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
