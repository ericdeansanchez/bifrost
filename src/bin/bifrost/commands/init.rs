//! Executes `bifrost init`.
use bifrost::core::config::Config;
use bifrost::ops::bifrost_init;
use bifrost::util::BifrostResult;

use clap::ArgMatches;

pub fn exec(config: Config, args: &ArgMatches) -> BifrostResult<()> {
    return bifrost_init::init(config, &args);
}
