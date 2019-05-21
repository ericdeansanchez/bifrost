use bifrost::ops::bifrost_init;
use bifrost::util::BifrostResult;
use bifrost::core::config::Config;

extern crate clap;
use clap::ArgMatches;

/// [NEED] parse args, update config if necessary...
pub fn exec(config: Config, args: &ArgMatches) -> BifrostResult<()> {
    return bifrost_init::init();
}
