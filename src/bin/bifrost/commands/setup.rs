//! Executes `bifrost setup`.
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

use bifrost::core::config::Config;
use bifrost::core::hofund;
use bifrost::util::docker::ImageBuilder;
use bifrost::util::process_builder::ProcessBuilder;
use bifrost::util::BifrostResult;

use clap::ArgMatches;

/// Executes the "setup" command which sets up the scaffolding necessary for
/// the other bifrost ops to be executed.
pub fn exec(config: Config, _args: &ArgMatches) -> BifrostResult<()> {
    if let Err(e) = check_installation(config.cwd()) {
        io::stderr().write_fmt(format_args!(
            "failed: `bifrost setup` could not find bifrost installation due to: {}\n",
            e,
        ))?;
        process::exit(1);
    }
    check_for_bifrost_directory(config.home_path())?;
    io::stdout().write("bifrost: building image - this could take a while...\n".as_bytes())?;
    setup(config.home_path())?;
    io::stdout().write("\nbifrost: successfully setup\n".as_bytes())?;
    Ok(())
}

/// Checks for a valid bifrost installation. Bifrost can be installed in a variety
/// of ways; as long as we have one we can execute the setup command.
fn check_installation(path: &PathBuf) -> BifrostResult<()> {
    let bifrost = ProcessBuilder {
        program: String::from("bifrost"),
        args: vec![String::from("--help")],
        cwd: Some(path.clone()),
    };

    let output = bifrost.exec()?;
    assert!(output.status.success());
    assert!(output.status.code() == Some(0i32));
    Ok(())
}

/// Checks for an existing bifrost directory. If found, this function alerts the
/// user and then exits. Prior to `bifrost setup` nothing should be set-up.
fn check_for_bifrost_directory(home_path: &PathBuf) -> BifrostResult<()> {
    if fs::metadata(&home_path.join(".bifrost")).is_ok() {
        io::stderr().write(
            "failed: `bifrost setup` cannot be run if bifrost has already been set up\n".as_bytes(),
        )?;
        process::exit(1);
    }
    // .bifrost directory does not exist and it is safe to
    // attempt to create it
    Ok(())
}

/// Sets up the bifrost scaffolding (i.e. bifrost and container directories).
fn setup(home_path: &PathBuf) -> BifrostResult<()> {
    create_bifrost_directory(home_path)?;
    build_image(home_path)?;
    Ok(())
}

/// Creates the necessary directory tree for bifrost to operate.
///
/// # Errors
///
/// If at any point this function cannot create a directory or cannot
/// write to the Dockerfile, this function returns an error.
fn create_bifrost_directory(home_path: &PathBuf) -> BifrostResult<()> {
    // Create the top-level hidden bifrost directory.
    let bifrost_path = home_path.join(".bifrost");
    fs::create_dir(&bifrost_path)?;

    // Create the container directory.
    let container_path = bifrost_path.join("container");
    fs::create_dir(&container_path)?;

    // Create the bifrost directory.
    let bifrost_container = container_path.join("bifrost");
    fs::create_dir(&bifrost_container)?;

    // Prepare the contents of the default Dockerfile.
    let docker_file_contents = r#"
FROM ubuntu:16.04

# Update default packages
RUN apt-get update

# Get Ubuntu packages
RUN apt-get install -y \
    build-essential \
    curl

# Update new packages
RUN apt-get update
"#;

    // Write the contents.
    let docker_file = bifrost_container.join("Dockerfile");
    hofund::write(&docker_file, docker_file_contents.as_bytes())?;
    Ok(())
}

/// Builds the docker image.
///
/// # Errors
///
/// The last step of this function is executes the build process. Building the
/// image can fail; if it does fail, then this function returns that error.
fn build_image(home_path: &PathBuf) -> BifrostResult<()> {
    let path = home_path.join(".bifrost").join("container").join("bifrost");

    let path = String::from(
        path.to_str()
            .expect("failed: `setup::build_image` failed to convert path to `str`"),
    );

    let image = ImageBuilder {
        name: String::from("docker"),
        tag: String::from("bifrost:0.1"),
        path,
    };

    Ok(image.build()?)
}

// Allow these tests to remain for documentation purposes. Eventually, they
// could/should be removed or transitioned into integration tests.
#[cfg(test)]
mod test {
    use super::*;
    use dirs;

    fn _test_check_installation() {
        let home_path = dirs::home_dir().expect(
            "error: `test_check_installation` expected
            home path to be `Some`",
        );
        assert!(check_installation(&home_path).is_ok());
    }

    fn _test_setup() {
        let home_path =
            dirs::home_dir().expect("error: `test_setup` expected home path to be `Some`");
        assert!(setup(&home_path).is_ok());
    }
}
