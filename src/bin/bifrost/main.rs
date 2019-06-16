use bifrost::core::app::cli;
use bifrost::core::config::Config;
use bifrost::util::BifrostResult;

pub mod commands;

fn main() -> BifrostResult<()> {
    // Open the bifrost.
    let app = cli().get_matches();

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
        _ => {
            println!("{}", bifrost::util::template::EXPLICIT_LONG_HELP);
        }
    }

    Ok(())
}
