use std::env;
use std::path::PathBuf;

use crate::config::Config;
use crate::dir::{self, WorkingDir};

#[derive(Debug)]
pub struct WorkSpace {
    pub workspace: WorkingDir,
    pub config: Config,
}

impl WorkSpace {
    pub fn init() -> WorkSpace {
        let path = dir::path_builder(dirs::home_dir(), ".bifrost/.config");
        let config = Config::init(path);

        let ignore_list: Vec<PathBuf> = vec![
            PathBuf::from(".git"),
            PathBuf::from("debug"),
            PathBuf::from("target"),
            PathBuf::from(".DS_Store"),
            PathBuf::from(".idea"),
        ];

        if let Ok(cwd) = env::current_dir() {
            let workspace = WorkingDir::init(Some(cwd), ignore_list);

            WorkSpace { config, workspace }
        } else {
            panic!("just going to panic for now");
        }
    }

    pub fn display(&self) {
        println!("{:#?}", self);
    }
}
