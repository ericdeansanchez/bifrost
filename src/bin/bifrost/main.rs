use bifrost::core::config::Config;
use bifrost::util::BifrostResult;
use bifrost::WorkSpace;

fn main() {
    // Open the bifrost.
    let app = bifrost::cli().get_matches();
    // config then,
    // ...,
    let mut config = Config::default();
    let mut workspace = WorkSpace::init(config);

    match app.subcommand() {
        _ => {
            println!("{}", bifrost::util::template::EXPLICIT_LONG_HELP);
        }
    }
}
