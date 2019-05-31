//! Implementation details of the `load` subcommand.

use crate::core::config::Config;
use crate::core::workspace::WorkSpace;
use crate::util::BifrostResult;
use crate::ArgMatches;

pub fn load(config: Config, args: &ArgMatches) -> BifrostResult<()> {
    let ws = WorkSpace::init(config, &args);
    println!("{:#?}", args);
    println!("{:#?}", ws);
    Ok(())
}

/*
fn load<P>(wd: &mut WorkingDir, to: P) -> Result<u64>
    where
        P: AsRef<Path>,
{
    let parent = wd
        .parent
        .as_ref()
        .map(|p| p.to_str().unwrap_or(""))
        .unwrap();

    // `dirs` will be empty after this if block.
    if let Some(from) = wd.dirs.pop() {
        if let Ok(path) = load_top_level_dir(&parent, &from, &to) {
            wd.unload_path = path;
        }
        while let Some(from) = wd.dirs.pop() {
            load_dir(&parent, &from, &to)?;
        }
    }
    return load_files(&parent, &wd, &to);
}

/// Load the top-level directory `to` its target destination. This will be the
/// path that this `WorkingDir` will be `unload`ed from.
fn load_top_level_dir<P>(parent: &str, from: &DirEntryExt, to: P) -> Result<UnloadPath>
    where
        P: AsRef<Path>,
{
    let mut t = to.as_ref().to_path_buf();
    let to_dir = from.0.path().clone();
    if let Ok(to_dir) = to_dir.strip_prefix(parent) {
        t.push(to_dir);
        fs::create_dir_all(&t)?;
        return Ok(UnloadPath(Some(t)));
    }
    Ok(UnloadPath(None))
}

fn load_dir<P>(parent: &str, from: &DirEntryExt, to: P) -> Result<()>
    where
        P: AsRef<Path>,
{
    let mut t = to.as_ref().to_path_buf();
    let to_dir = from.0.path().clone();
    if let Ok(to_dir) = to_dir.strip_prefix(parent) {
        t.push(to_dir);
        fs::create_dir_all(&t)?;
    }
    Ok(())
}

fn load_files<P>(parent: &str, wd: &WorkingDir, to: P) -> Result<u64>
    where
        P: AsRef<Path>,
{
    let mut nbytes = 0;
    for entry in &wd.files {
        let to = to.as_ref().to_path_buf();

        if let Ok(to) = entry.strip_prefix(parent).map(|e| to.join(e)) {
            File::create(&to)?;
            if let Ok(n) = fs::copy(entry, to) {
                nbytes += n;
            }
        }
    }
    Ok(nbytes)
}

fn unload(path: UnloadPath) -> Result<()> {
    unimplemented!();
}
*/
