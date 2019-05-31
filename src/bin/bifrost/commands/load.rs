use bifrost::core::config::Config;
use bifrost::ops::bifrost_load;
use bifrost::util::BifrostResult;

use clap::ArgMatches;

pub fn exec(config: Config, args: &ArgMatches) -> BifrostResult<()> {
    let config = config.config_manifest(&args);
    return bifrost_load::load(config, &args);
}
