//! Implementation details of the `init` subcommand.
use std::fs;
use std::io::{self, Write};

use crate::core::config::Config;
use crate::core::hofund;
use crate::util::BifrostResult;
use crate::ArgMatches;

/// Initialize a given directory as a Bifrost realm.
///
/// If `args` has no `ArgMatches`, then the default TOML string is written to
/// the Bifrost.toml manifest.
///
/// If `args` does have `ArgMatches`, then the `Config`'s manifest is configured
/// with these arguments.
///
/// If the current `Config`'s manifest is `None`, then the default TOML string is
/// written to the Bifrost.toml manifest. Otherwise, the `Config`'s newly configured
/// manifest is converted `to_str` to be written.
///
/// If for some reason the `Config`'s `BifrostManifest` cannot be converted `to_str`,
/// then the default TOML string is written to the Bifrost.toml manifest.
///
/// # Errors
///
/// If an existing Bifrost.toml manifest is found in the current working
/// directory, then this function assumes the directory has already been
/// initialized.
///
/// This function can also return an error if any call to `hofund::write`
/// results in an error.
pub fn init(config: Config, args: &ArgMatches) -> BifrostResult<()> {
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

    if fs::metadata(&config.cwd().join("Bifrost.toml")).is_ok() {
        failure::bail!("error: `bifrost init` cannot be run on an existing Bifrost realm")
    }

    let success = |p: &std::path::PathBuf| -> BifrostResult<()> {
        io::stdout().write_fmt(format_args!(
            "Initialized default Bifrost realm in {}\n",
            p.display()
        ))?;
        Ok(())
    };

    if args.args.is_empty() {
        hofund::write(&config.cwd().join("Bifrost.toml"), &toml.as_bytes())?;
        success(config.cwd())?;
    } else {
        let config = config.config_manifest(&args);
        let toml = match &config.manifest() {
            Some(s) => match s.to_str() {
                Ok(s) => s,
                Err(e) => {
                    io::stderr().write_fmt(format_args!(
                        "warn: failed to convert Bifrost manifest \
                         to string due to `{}` used default instead",
                        e
                    ))?;

                    String::from(toml)
                }
            },
            None => String::from(toml),
        };

        hofund::write(&config.cwd().join("Bifrost.toml"), toml.as_bytes())?;
        success(config.cwd())?;
    }

    Ok(())
}
