use failure;
use std::fmt::{self, Debug};

pub type BifrostResult<T> = failure::Fallible<T>;

pub struct OperationInfo {
    pub name: String,
    pub bytes: Option<u64>,
    pub text: Option<Vec<u8>>,
}

impl OperationInfo {
    pub fn new() -> Self {
        OperationInfo {
            ..Default::default()
        }
    }
}

impl Default for OperationInfo {
    fn default() -> Self {
        OperationInfo {
            name: String::new(),
            bytes: None,
            text: None,
        }
    }
}

impl Debug for OperationInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WorkingDir")
            .field("workspace", &self.name)
            .field("size", &self.bytes)
            .field("text", &self.text)
            .finish()
    }
}

#[derive(Debug)]
pub struct BifrostOptions {
    pub verbose: bool,
    pub diff: bool,
    pub max_depth: u64,
}

impl Default for BifrostOptions {
    fn default() -> Self {
        BifrostOptions {
            verbose: false,
            diff: false,
            max_depth: 0u64,
        }
    }
}

#[macro_use]
mod macros;

pub mod bifrost_path;
pub mod error;
pub mod template;

pub use bifrost_path::BifrostPath;
