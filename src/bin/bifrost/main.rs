use bifrost::core::config::Config;
use bifrost::util::BifrostResult;
use bifrost::WorkSpace;

pub mod commands;

fn main() -> BifrostResult<()> {
    // Open the bifrost.
    let app = bifrost::cli().get_matches();
    let config = Config::default();

    match app.subcommand() {
        ("init", Some(arg_matches)) => {
            commands::init::exec(config, arg_matches)?;
        }
        _ => {
            println!("{}", bifrost::util::template::EXPLICIT_LONG_HELP);
        }
    }

    Ok(())
}
