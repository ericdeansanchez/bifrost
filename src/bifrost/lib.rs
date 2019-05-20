//!
#[macro_use]
extern crate clap;

pub use crate::core::app::cli;
pub use crate::core::workspace::WorkSpace;
pub use crate::util::BifrostResult;

pub mod core;
pub mod ops;
pub mod util;
