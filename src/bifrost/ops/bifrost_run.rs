//! Implementation details of the `run` subcommand.
use std::fs;
use std::io::{self, Write};
use std::process;

use crate::core::config::Config;
use crate::core::workspace::{BifrostOperable, WorkSpace};
use crate::util::{bifrost_path, BifrostResult, OperationInfo};
use crate::ArgMatches;

pub fn run(config: Config, args: &ArgMatches) -> BifrostResult<()> {
    let success = |op_info: OperationInfo| -> BifrostResult<()> {
        io::stdout().write_fmt(format_args!(
            "bifrost: workspace realm {{{}}}\nstdout:\n{}\nstderr:\n{}",
            op_info.name, op_info.stdout, op_info.stderr,
        ))?;
        Ok(())
    };

    // Construct the `RunSpace`.
    let mut ws = WorkSpace::to_run_space(config, &args);

    // Prepare the `RunSpace`.
    let ws = ws.prep()?;

    // Get the `BifrostPath`'s underlying `PathBuf`.
    let path = bifrost_path::get_path_or_empty(ws.target());

    // If the path does not exist, then...
    if fs::metadata(&path).is_err() {
        // [TODO] search similar path names and attempt to resolve manifest
        // modifications.
        let path = bifrost_path::handle_bad_path(path);
        io::stdout().write_fmt(format_args!(
            "failed: to `run` {{{}}} are you sure you have called `bifrost load`?\n",
            path
        ))?;
        process::exit(1);
    }

    let op_info = ws.build()?.exec()?;
    return Ok(success(op_info)?);
}
