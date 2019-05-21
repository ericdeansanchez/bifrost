use std::env;

use crate::core::hofund;
use crate::util::BifrostResult;

/// [NEED] Handle existing Bifrost.toml case(s).
pub fn init() -> BifrostResult<()> {
    let cwd = env::current_dir()?;
    let toml: &str = r#"[project]
name = "project name"

[container]
name = "docker"

[workspace]
name = "name of workspace"
ignore = ["target", ".git", ".gitignore"]

[command]
cmd = ["command string(s)"]
"#;

    let cwd = cwd.join("Bifrost.toml");
    hofund::write(cwd.as_ref(), toml.as_bytes())?;
    println!(
        "{}",
        format!("Initialized default Bifrost realm in {}", cwd.display())
    );
    Ok(())
}

/*
#[cfg(test)]
mod test {
    use super::*;

    // [BREAK] This will break travis.
    // [NEED] Install scripts.
    #[test]
    fn write_to_tmp() -> BifrostResult<()> {
        (|| -> BifrostResult<()> {
            let mut target = dirs::home_dir().unwrap();
            target.push(".bifrost");
            target.push("tmp");

            let toml: &str = r#"[project]
name = "project name"

[container]
name = "docker"

[workspace]
name = "name of workspace"
ignore = ["target", ".git", ".gitignore"]

[command]
cmd = ["command string(s)"]
"#;

            let target = target.join("Bifrost.toml");
            hofund::write(target.as_ref(), toml.as_bytes())?;
            Ok(())
        })()
    }

    #[cfg(not(linux))]
    #[test]
    fn remove_from_tmp() -> BifrostResult<()> {
        // To ensure Bifrost.toml is written before removal;
        // tests don't necessarily run sequentially.
        write_to_tmp()?;

        (|| -> BifrostResult<()> {
            let mut target = dirs::home_dir().unwrap();
            target.push(".bifrost");
            target.push("tmp");
            let target = target.join("Bifrost.toml");
            hofund::remove_file(target.as_ref())?;
            Ok(())
        })()
    }
}
*/
