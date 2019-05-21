// Hofund is the sword Heimdallr uses to open the Bifrost.
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

use crate::util::BifrostResult;

pub fn write(path: &Path, contents: &[u8]) -> BifrostResult<()> {
    let mut f = File::create(path)?;
    f.write_all(contents)?;
    Ok(())
}

pub fn append(path: &Path, contents: &[u8]) -> BifrostResult<()> {
    let mut f = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)?;

    f.write_all(contents)?;
    Ok(())
}
