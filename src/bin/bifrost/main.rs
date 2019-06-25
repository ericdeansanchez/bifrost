use bifrost::core::app::cli;
use bifrost::core::config::Config;
use bifrost::util::{docker, BifrostResult};

use std::io::{self, Write};
use std::process;

pub mod commands;

fn main() -> BifrostResult<()> {
    // Open the bifrost.

    // [TODO] Look into `get_matches_safe`
    let app = cli().get_matches();

    // [TODO] Looks like the only thing `exec` needs is `arg_matches`
    match app.subcommand() {
        ("init", Some(arg_matches)) => {
            commands::init::exec(Config::default(), arg_matches)?;
        }
        ("load", Some(arg_matches)) => {
            commands::load::exec(Config::default(), arg_matches)?;
        }
        ("show", Some(arg_matches)) => {
            commands::show::exec(Config::default(), arg_matches)?;
        }
        ("unload", Some(arg_matches)) => {
            commands::unload::exec(Config::default(), arg_matches)?;
        }
        ("run", Some(arg_matches)) => {
            let config = Config::default();
            // TODO - determine best way of starting docker...

            if !docker::is_installed() {
                io::stderr().write(
                    "error: cannot `run` is docker installed (e.g. $ docker -v)".as_bytes(),
                )?;
                process::exit(1);
            }

            if !docker::is_running() {
                docker::start(config.home_path())?;
            }
            commands::run::exec(config, arg_matches)?;
        }
        _ => {
            println!("{}", bifrost::util::template::EXPLICIT_LONG_HELP);
        }
    }

    let one = std::time::Duration::from_secs(1);
    std::thread::sleep(one);

    Ok(())
}
