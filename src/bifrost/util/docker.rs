//! Structures and functions for working with docker.
use crate::core::hofund;
use crate::util::{BifrostResult, ProcessBuilder};

use std::path::Path;

use dirs;
use spinner::SpinnerBuilder;

/// Checks whether or not docker is currently installed.
pub fn is_installed() -> bool {
    let version = ProcessBuilder {
        program: String::from("docker"),
        args: vec![String::from("-v")],
        cwd: dirs::home_dir(),
    };

    let output = version.exec();
    match output {
        Err(_) => false,
        Ok(o) => {
            if !o.stderr.is_empty() || o.status.code() != Some(0i32) {
                return false;
            }
            return true;
        }
    }
}

/// Checks whether or not docker is currently up and running.
pub fn is_running() -> bool {
    let info = ProcessBuilder {
        program: String::from("docker"),
        args: vec![String::from("system"), String::from("info")],
        cwd: dirs::home_dir(),
    };
    return info.exec().is_ok();
}

/// Starts docker in the background.
///
/// # Errors
///
/// If the process failed to execute or if the process was executed, but
/// an error occurred during execution, then this function returns an
/// error.
pub fn start<P: AsRef<Path>>(home: &P) -> BifrostResult<Option<i32>> {
    let open = ProcessBuilder {
        program: String::from("open"),
        args: vec![
            String::from("--background"),
            String::from("-a"),
            String::from("Docker"),
        ],

        cwd: Some(home.as_ref().to_path_buf()),
    };

    let output = open.exec()?;
    if !output.status.success() {
        failure::bail!(
            "error: failed to start docker with status: {:#?}",
            output.status.code()
        );
    }

    {
        let _sp = SpinnerBuilder::new("Starting Docker...".into())
            .spinner(vec![
                "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏",
            ])
            .start();
        while !is_running() {}
    }

    Ok(output.status.code())
}

/// Stops docker by looking for its PID and killing that process.
///
/// # Errors
///
/// This method errors if any of the following fail:
///
/// * getting the docker pid
/// * if the process executing the above function returns un`success`fully
/// * conversion from a `Vec<u8>` to `String::from_utf8` fails
/// * killing the docker process failed
pub fn stop() -> BifrostResult<()> {
    let get_docker_pid = ProcessBuilder {
        program: String::from("bash"),
        args: vec![String::from("-c"), String::from("printf $(pgrep Docker)")],
        cwd: None,
    };

    let output = get_docker_pid.exec()?;
    if !output.status.success() {
        failure::bail!(
            "error: failed to `get_docker_pid` docker with status: {}",
            output.status.code().unwrap_or(1)
        );
    }

    let pid = String::from_utf8(output.stdout)?;

    let kill = ProcessBuilder {
        program: String::from("kill"),
        args: vec![pid],
        cwd: None,
    };

    let output = kill.exec()?;
    if !output.status.success() {
        failure::bail!(
            "error: failed to `kill` docker with status: {}",
            output.status.code().unwrap_or(1)
        );
    };
    Ok(())
}

/// A near-transliteration of linux's apt-get. That is, the name
/// of this function is nearly what will appear in the Dockerfile.
///
/// For example, calling this function with the name 'app' translates
/// to 'RUN apt-get install build-essential app -y'. This function provides
/// a way of adding to the Dockerfile.
///
/// # Errors
///
/// If `hofund::append` fails to append to the Dockerfile, this function
/// returns the error produced by that failure.
pub fn apt_get_install_build_essential(name: &str) -> BifrostResult<()> {
    if let Some(path) = dirs::home_dir() {
        let docker_file = path
            .join(".bifrost")
            .join("container")
            .join("bifrost")
            .join("Dockerfile");
        let contents = format!(
            r#"

# Bifrost Appended:
RUN apt-get install build-essential {} -y

"#,
            name
        );
        hofund::append(&docker_file, &contents.as_bytes())?;
    }
    Ok(())
}

/// Primary structure for dealing with images.
pub struct ImageBuilder {
    /// The name of the container engine.
    pub name: String,
    /// The tag that this image gets built with.
    pub tag: String,
    /// The path to the mounted directory.
    pub path: String,
}

impl ImageBuilder {
    /// Builds the docker image by executing a process.
    ///
    /// # Errors
    ///
    /// This method returns an error if the process execution
    /// fails or the process returns an error itself.
    /// If the process executed successfully, but the status
    /// output by this execution indicates its function was
    /// unsuccessful, then this method returns an error.
    pub fn build(&self) -> BifrostResult<()> {
        let docker_process = ProcessBuilder {
            program: self.name.clone(),
            args: vec![
                String::from("build"),
                String::from("-t"),
                self.tag.clone(),
                self.path.clone(),
            ],
            cwd: None,
        };

        let _sp = SpinnerBuilder::new("Building Image...".into())
            .spinner(vec![
                "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏",
            ])
            .start();

        match docker_process.exec() {
            Ok(output) => {
                if !output.status.success() {
                    failure::bail!(
                        "failed: `ImageBuilder::build` failed to successfully build the image"
                    );
                }
            }
            Err(e) => {
                failure::bail!("error: failed to build image due to {}", e);
            }
        }
        Ok(())
    }
}

// Allow these tests to remain for documentation purposes. Eventually, they
// could/should be removed or transitioned into integration tests.
#[cfg(test)]
mod test {
    use super::*;

    fn _start_bifrost_container<P>(path: &P) -> BifrostResult<Option<i32>>
    where
        P: AsRef<Path>,
    {
        let mounted_dir = path
            .as_ref()
            .join(".bifrost")
            .join("container")
            .join("test");

        let mounted_str = mounted_dir.to_str().expect("BUG: ...");

        let process = ProcessBuilder {
            program: String::from("docker"),
            args: vec![
                String::from("--version"),
                String::from("run"),
                String::from("--rm"),
                String::from("-i"),
                String::from("-v"),
                String::from(mounted_str),
                String::from("bifrost:0.1"),
            ],

            cwd: Some(path.as_ref().to_path_buf()),
        };

        Ok(process.exec()?.status.code())
    }

    fn _test_with_docker() -> BifrostResult<()> {
        use crate::util::docker;

        let home_path =
            dirs::home_dir().expect("could not get home path will testing `process_builder`");

        if docker::is_running() {
            let code = _start_bifrost_container(&home_path)?;
            assert_eq!(Some(0i32), code);
        } else {
            docker::start(&home_path)?;
            let code = _start_bifrost_container(&home_path)?;
            assert_eq!(Some(0i32), code);
            docker::stop()?;
        }
        Ok(())
    }

    fn _test_docker_is_installed() -> BifrostResult<()> {
        // Will fail as long as Docker is not installed.
        assert_eq!(true, is_installed());
        Ok(())
    }

    fn _test_image_build() {
        let path = dirs::home_dir()
            .expect("failed: `test_image_build` expected home path to be `Some`")
            .join(".bifrost")
            .join("container")
            .join("bifrost");

        let path = String::from(
            path.to_str()
                .expect("failed: `setup::build_image` failed to convert path to `str`"),
        );

        let image = ImageBuilder {
            name: String::from("docker"),
            tag: String::from("bifrost:0.1"),
            path,
        };

        assert!(image.build().is_ok());
    }
}
