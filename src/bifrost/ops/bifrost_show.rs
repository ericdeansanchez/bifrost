//! Implementation details of the `show` subcommand.
use std::fs;
use std::io::{self, Write};
use std::process;

use crate::core::config::Config;
use crate::core::workspace::{BifrostOperable, WorkSpace};
use crate::util::{bifrost_path, BifrostResult, OperationInfo};
use crate::ArgMatches;

pub fn show(config: Config, args: &ArgMatches) -> BifrostResult<()> {
    let success = |op_info: OperationInfo| -> BifrostResult<()> {
        io::stdout().write_fmt(format_args!(
            "bifrost: workspace realm {{{}}} {}",
            op_info.name,
            String::from_utf8_lossy(&op_info.text.unwrap())
        ))?;
        Ok(())
    };

    // Construct the `ShowSpace`.
    let mut ws = WorkSpace::to_show_space(config, &args);

    // Prepare the `ShowSpace`.
    let ws = ws.prep()?;

    // Get the `BifrostPath`'s underlying `PathBuf`.
    let path = bifrost_path::get_path_or_empty(ws.bifrost_path());

    // If the path does not exist, then...
    if fs::metadata(&path).is_err() {
        // [TODO] search similar path names and attempt to resolve manifest
        // modifications.
        let path = bifrost_path::handle_bad_path(path);
        io::stdout().write_fmt(format_args!(
            "failed: to `show` {{{}}} are you sure you have called `bifrost load`?\n",
            path
        ))?;
        process::exit(1);
    }

    let op_info = ws.build()?.exec()?;
    return Ok(success(op_info)?);
}
