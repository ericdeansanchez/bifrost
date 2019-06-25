use failure;

pub type BifrostResult<T> = failure::Fallible<T>;

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
pub mod docker;
pub mod error;
pub mod operation_info;
pub mod process_builder;
pub mod template;

pub use bifrost_path::BifrostPath;
pub use operation_info::OperationInfo;
pub use process_builder::ProcessBuilder;
