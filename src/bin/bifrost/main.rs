use bifrost::core::config::Config;
use bifrost::util::BifrostResult;
use bifrost::WorkSpace;

pub mod commands;

fn main() -> BifrostResult<()> {
    // Open the bifrost.
    let app = bifrost::cli().get_matches();
    // config then,
    // ...,
    let config = Config::default();
    let workspace = WorkSpace::init(config);

    match app.subcommand() {
        ("init", Some(init_matches)) => {
            commands::init::exec()?;
        }
        _ => {
            println!("{}", bifrost::util::template::EXPLICIT_LONG_HELP);
        }
    }

    Ok(())
}
