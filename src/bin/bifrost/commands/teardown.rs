//! Executes `bifrost setup`.
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

use bifrost::core::config::Config;
use bifrost::core::hofund;
use bifrost::util::BifrostResult;

use clap::ArgMatches;

/// Executes the "teardown" command which tears down the scaffolding necessary for
/// the other bifrost ops to be executed. This command performs the reciprocal
/// operation of `bifrost setup`.
pub fn exec(config: Config, _args: &ArgMatches) -> BifrostResult<()> {
    io::stdout().write("bifrost: tearing down bifrost directory... ".as_bytes())?;
    teardown(config.home_path())?;
    io::stdout().write_all("done\n".as_bytes())?;
    Ok(())
}

/// Checks that the dot-bifrost directory exists and removes it if it does exist;
/// Otherwise, a message is written to stderr alerting the user that it could not
/// find the dot-bifrost file where it expected to (i.e. $HOME/.bifrost).
///
/// It is hard to say under what conditions the dot-bifrost won't be found; if it
/// is removed bifrost assumes that it has not been setup and will prompt the user
/// to call `bifrost setup`. This means, that the dot-bifrost directory will go
/// right where it was expected to be.
fn teardown(home_path: &PathBuf) -> BifrostResult<()> {
    let dot_bifrost = home_path.join(".bifrost");
    if fs::metadata(&dot_bifrost).is_err() {
        let path = dot_bifrost
            .to_str()
            .expect("BUG: `teardown` expected home path to be `Some`");
        io::stderr().write_fmt(format_args!(
            "failed: `bifrost teardown`failed to find bifrost directory at {}\n",
            path,
        ))?;
        process::exit(1);
    }
    // Remove the directory.
    hofund::remove_dir_all(&dot_bifrost)?;
    Ok(())
}
