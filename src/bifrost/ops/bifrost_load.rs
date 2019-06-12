//! Implementation details of the `load` subcommand.
use std::io::{self, Write};

use crate::core::config::Config;
use crate::core::workspace::{BifrostOperable, WorkSpace};
use crate::util::{BifrostResult, OperationInfo};
use crate::ArgMatches;

/// Loads a Bifrost Workspace. Utilizes `Config` and `ArgMatches` to construct
/// a `WorkSpace`. This workspace is then loaded to the target directory specified
/// by a `BifrostPath`. Here, **load** is synonymous with **copy**.
pub fn load(config: Config, args: &ArgMatches) -> BifrostResult<()> {
    let success = |op_info: &OperationInfo| -> BifrostResult<()> {
        io::stdout().write_fmt(format_args!(
            "loaded `{}` bytes from workspace realm `{}`\n",
            op_info.bytes.unwrap(),
            op_info.name,
        ))?;
        Ok(())
    };

    let op_info = WorkSpace::to_load_space(config, &args)
        .prep()?
        .build()?
        .exec()?;

    success(&op_info)?;
    return Ok(());
}
