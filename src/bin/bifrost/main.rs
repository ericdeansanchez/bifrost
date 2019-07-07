use bifrost::core::app::cli;
use bifrost::core::config::Config;
use bifrost::util::{docker, BifrostResult};

use std::io::{self, Write};
use std::path::Path;
use std::process;

pub mod commands;

fn main() -> BifrostResult<()> {
    // Open the bifrost.
    // [TODO] Look into `get_matches_safe`
    let app = cli().get_matches();

    let config = Config::default();

    // [TODO] Looks like the only thing `exec` needs is `arg_matches`
    // [TODO] Clean this repetition up
    match app.subcommand() {
        ("init", Some(arg_matches)) => {
            exit_if_not_setup(&config)?;
            commands::init::exec(config, arg_matches)?;
        }
        ("load", Some(arg_matches)) => {
            exit_if_not_setup(&config)?;
            exit_if_uninitialized(&config, "load")?;
            commands::load::exec(config, arg_matches)?;
        }
        ("show", Some(arg_matches)) => {
            exit_if_not_setup(&config)?;
            exit_if_uninitialized(&config, "show")?;
            commands::show::exec(config, arg_matches)?;
        }
        ("unload", Some(arg_matches)) => {
            exit_if_not_setup(&config)?;
            exit_if_uninitialized(&config, "unload")?;
            commands::unload::exec(config, arg_matches)?;
        }
        ("run", Some(arg_matches)) => {
            exit_if_not_setup(&config)?;
            exit_if_uninitialized(&config, "run")?;
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
        ("setup", Some(arg_matches)) => {
            commands::setup::exec(config, arg_matches)?;
        }
        _ => {
            println!("{}", bifrost::util::template::EXPLICIT_LONG_HELP);
        }
    }
    Ok(())
}

fn exit_if_not_setup(config: &Config) -> BifrostResult<()> {
    let set_up = check_setup(config.home_path());
    let alert_not_setup = || -> BifrostResult<()> {
        io::stderr().write(
            r#"
bifrost: Congrats on installing bifrost! There is just one more
         step to complete the installation, run:

            $ bifrost setup
         
         this command will create some directories and files
         that bifrost needs to do its thing. To learn more,
         see https://github.com/ericdeansanchez/bifrost

"#
            .as_bytes(),
        )?;
        Ok(())
    };

    if !set_up {
        alert_not_setup()?;
        process::exit(1);
    }

    Ok(())
}

fn check_setup<P: AsRef<Path>>(path: P) -> bool {
    use std::fs;
    if fs::metadata(&path.as_ref().join(".bifrost")).is_err() {
        return false;
    }
    true
}

fn exit_if_uninitialized(config: &Config, op: &str) -> BifrostResult<()> {
    let initialized = check_initialized(&config.cwd());
    let alert_uninitialized = |op: &str| -> BifrostResult<()> {
        io::stderr().write_fmt(format_args!(
            "failed: bifrost failed to {} because realm has not been initialized\n",
            op,
        ))?;
        Ok(())
    };

    if !initialized {
        alert_uninitialized(op)?;
        process::exit(1);
    }
    Ok(())
}

pub fn check_initialized<P: AsRef<Path>>(cwd: P) -> bool {
    use std::fs;
    if fs::metadata(&cwd.as_ref().join("Bifrost.toml")).is_err() {
        return false;
    }
    true
}
