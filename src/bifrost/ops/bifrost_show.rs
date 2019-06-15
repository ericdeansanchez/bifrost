//! Implementation details of the `show` subcommand.
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::core::config::Config;
use crate::core::workspace::{BifrostOperable, WorkSpace};
use crate::util::{BifrostPath, BifrostResult, OperationInfo};
use crate::ArgMatches;

pub fn show(config: Config, args: &ArgMatches) -> BifrostResult<()> {
    let success = |op_info: OperationInfo| -> BifrostResult<()> {
        io::stdout().write_fmt(format_args!(
            "\nbifrost-realm `{}`: {}\n",
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
    let path = get_path_or_panic(ws.bifrost_path());

    // If the path does not exist, then...
    if fs::metadata(&path).is_err() {
        // handle this case
    }

    let op_info = ws.build()?.exec()?;
    success(op_info)?;
    return Ok(());
}

fn get_path_or_panic(maybe_path: Option<BifrostPath>) -> PathBuf {
    maybe_path
        .expect("BUG: `bifrost_show::show` failed because it expected `BifrostPath` to be `Some`")
        .path
}
