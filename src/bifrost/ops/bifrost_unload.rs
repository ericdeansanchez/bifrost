//! Implementation details of the `show` subcommand.
use std::fs;
use std::io::{self, Write};
use std::process;

use crate::core::config::Config;
use crate::core::workspace::{BifrostOperable, WorkSpace};
use crate::util::{bifrost_path, BifrostResult, OperationInfo};
use crate::ArgMatches;

pub fn unload(config: Config, args: &ArgMatches) -> BifrostResult<()> {
    let success = |op_info: OperationInfo| -> BifrostResult<()> {
        io::stdout().write_fmt(format_args!(
            "bifrost: successfully unloaded workspace realm `{}`\n",
            op_info.name,
        ))?;
        Ok(())
    };

    let mut ws = WorkSpace::to_unload_space(config, &args);
    let ws = ws.prep()?;

    // Get the `BifrostPath`'s underlying `PathBuf`.
    let path = bifrost_path::get_path_or_panic(ws.bifrost_path());

    // If the path does not exist, then...
    if fs::metadata(&path).is_err() {
        let path = bifrost_path::handle_bad_path(path);

        io::stdout().write_fmt(format_args!(
            "failed: to `unload` {{{}}} are you sure you have called `bifrost load`?\n",
            path
        ))?;

        process::exit(1);
    }

    let op_info = ws.build()?.exec()?;
    success(op_info)?;
    return Ok(());
}
