use bifrost::ops::bifrost_init;
use bifrost::util::BifrostResult;

pub fn exec() -> BifrostResult<()> {
    return bifrost_init::init();
}
