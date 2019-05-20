use failure;

pub type BifrostResult<T> = failure::Fallible<T>;

#[macro_use]
mod macros;

pub mod error;
pub mod template;
