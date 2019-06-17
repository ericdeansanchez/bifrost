//! Implementation details of the `run` subcommand.
use std::io::{self, Write};

use crate::core::config::Config;
use crate::core::workspace::{BifrostOperable, WorkSpace};
use crate::util::{BifrostResult, OperationInfo};
use crate::ArgMatches;

pub fn run(config: Config, args: &ArgMatches) -> BifrostResult<()> {
    return Ok(());
}
