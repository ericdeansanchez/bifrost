use std::path::PathBuf;

use crate::core::config::Config;
use crate::core::working_dir::WorkingDir;

#[derive(Debug)]
pub struct WorkSpace {
    config: Config,
    registered_paths: Vec<PathBuf>,
    working_dirs: Vec<WorkingDir>,
}

impl Default for WorkSpace {
    fn default() -> Self {
        WorkSpace {
            config: Config::default(),
            registered_paths: vec![],
            working_dirs: vec![],
        }
    }
}

impl WorkSpace {
    pub fn init(config: Config) -> Self {
        WorkSpace {
            config,
            ..Default::default()
        }
    }
}
