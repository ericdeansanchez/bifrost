use crate::util::{BifrostResult, ProcessBuilder};

use std::path::Path;

use dirs;
use spinner::SpinnerBuilder;

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

pub fn is_running() -> bool {
    let info = ProcessBuilder {
        program: String::from("docker"),
        args: vec![String::from("system"), String::from("info")],
        cwd: dirs::home_dir(),
    };
    return info.exec().is_ok();
}

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

// [FIX ME] move these tests to be integration tests.
#[cfg(test)]
mod test {
    use super::*;

    fn start_bifrost_container<P>(path: &P) -> BifrostResult<Option<i32>>
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

    #[test]
    fn test_with_docker() -> BifrostResult<()> {
        use crate::util::docker;

        let home_path =
            dirs::home_dir().expect("could not get home path will testing `process_builder`");

        if docker::is_running() {
            let code = start_bifrost_container(&home_path)?;
            assert_eq!(Some(0i32), code);
        } else {
            docker::start(&home_path)?;
            let code = start_bifrost_container(&home_path)?;
            assert_eq!(Some(0i32), code);
            docker::stop()?;
        }
        Ok(())
    }

    #[test]
    fn test_docker_is_installed() -> BifrostResult<()> {
        // Will fail as long as Docker is not installed.
        assert_eq!(true, is_installed());
        Ok(())
    }
}
