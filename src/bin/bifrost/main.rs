use bifrost::core::app::cli;
use bifrost::core::config::Config;
use bifrost::util::{docker, BifrostResult};

use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process;

pub mod commands;

fn main() -> BifrostResult<()> {
    // Open the bifrost.
    // [TODO] Look into `get_matches_safe`
    let app = cli().get_matches();

    // [TODO] Looks like the only thing `exec` needs is `arg_matches`
    // [TODO] Clean this repetition up
    match app.subcommand() {
        ("init", Some(arg_matches)) => {
            let config = Config::default();
            exit_if_not_setup(&config)?;
            commands::init::exec(config, arg_matches)?;
        }
        ("load", Some(arg_matches)) => {
            let config = Config::default();
            exit_if_not_setup(&config)?;
            exit_if_uninitialized(&config, "load")?;
            commands::load::exec(config, arg_matches)?;
        }
        ("show", Some(arg_matches)) => {
            let config = Config::default();
            exit_if_not_setup(&config)?;
            exit_if_uninitialized(&config, "show")?;
            commands::show::exec(config, arg_matches)?;
        }
        ("unload", Some(arg_matches)) => {
            let config = Config::default();
            exit_if_not_setup(&config)?;
            exit_if_uninitialized(&config, "unload")?;
            commands::unload::exec(config, arg_matches)?;
        }
        ("run", Some(arg_matches)) => {
            let config = Config::default();
            exit_if_not_setup(&config)?;
            exit_if_uninitialized(&config, "run")?;
            start_container_or_exit(config.home_path())?;
            commands::run::exec(config, arg_matches)?;
        }
        ("setup", Some(arg_matches)) => {
            // Construct a new `Config`--a default config instance cannot be
            // constructed in the top-level $HOME directory (as it assumes this
            // isn't what a user intends).
            let cwd = match env::current_dir() {
                Ok(path) => Some(path),
                Err(e) => {
                    io::stderr().write_fmt(format_args!(
                        "failed: `bifrost setup` failed to get the \
                         current working directory due to: {}",
                        e
                    ))?;
                    process::exit(1);
                }
            };

            let config = Config::new(dirs::home_dir(), cwd);
            start_container_or_exit(config.home_path())?;
            commands::setup::exec(config, arg_matches)?;
        }
        ("teardown", Some(arg_matches)) => {
            let config = Config::default();
            exit_if_not_setup(&config)?;
            io::stdout()
                .write_all("bifrost: are you sure you want to teardown? y or n ".as_bytes())?;
            io::stdout().flush()?;

            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    if let Some(c) = input.chars().next() {
                        if c == 'y' || c == 'Y' {
                            commands::teardown::exec(config, arg_matches)?;
                        }
                    }
                }
                Err(e) => {
                    io::stderr().write_fmt(format_args!(
                        "failed: `bifrost teardown` encountered the following error: {}",
                        e
                    ))?;
                    process::exit(1);
                }
            }
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

fn check_initialized<P: AsRef<Path>>(cwd: P) -> bool {
    use std::fs;
    if fs::metadata(&cwd.as_ref().join("Bifrost.toml")).is_err() {
        return false;
    }
    true
}

fn start_container_or_exit<P: AsRef<Path>>(path: P) -> BifrostResult<()> {
    if !docker::is_installed() {
        io::stderr().write(
            "\
failed: bifrost failed to start container
    
    is docker installed (e.g. $ docker -v)?
"
            .as_bytes(),
        )?;
        process::exit(1);
    }
    if !docker::is_running() {
        docker::start(&path)?;
    }
    Ok(())
}
