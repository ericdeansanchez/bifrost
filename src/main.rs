extern crate bifrost;
extern crate clap;

use bifrost::workspace::WorkSpace;

fn main() {
    // Open the bifrost.
    let app = bifrost::app::app().get_matches();
    let workspace = WorkSpace::init();

    match app.subcommand() {
        ("load", Some(contents)) => {
            bifrost::command::load(&workspace, contents);
        }
        ("unload", Some(unload_matches)) => {
            bifrost::command::unload();
        }
        ("show", Some(show_matches)) => {
            bifrost::command::show(&workspace);
        }
        ("run", Some(run_matches)) => {
            bifrost::command::run();
        }
        _ => {
            println!("{}", bifrost::template::EXPLICIT_LONG_HELP);
        }
    }
}
