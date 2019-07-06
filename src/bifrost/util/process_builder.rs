//! Data structures, methods, and functions for building processes.
//! This code is heavily inspired by [cargo's](https://github.com/rust-lang/cargo) own
//! process builder that can be found [here](https://github.com/rust-lang/cargo/blob/master/src/cargo/util/process_builder.rs)
use std::fmt;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output};
use std::str;

#[derive(Clone, Debug)]
pub struct ProcessBuilder {
    /// The program to be executed from within the Bifrost container.
    pub program: String,
    /// The arguments supplied to the program to be executed.
    pub args: Vec<String>,
    /// The directory from which to execute the given program.
    pub cwd: Option<PathBuf>,
}

impl ProcessBuilder {
    pub fn program<T: AsRef<str>>(&mut self, program: T) -> &mut ProcessBuilder {
        self.program = program.as_ref().to_string();
        self
    }

    pub fn arg<T: AsRef<str>>(&mut self, arg: T) -> &mut ProcessBuilder {
        self.args.push(arg.as_ref().to_string());
        self
    }

    pub fn cwd<T: AsRef<Path>>(&mut self, path: T) -> &mut ProcessBuilder {
        self.cwd = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn get_program(&self) -> &String {
        &self.program
    }

    pub fn get_args(&self) -> &[String] {
        &self.args
    }

    pub fn get_cwd(&self) -> Option<&PathBuf> {
        self.cwd.as_ref()
    }

    pub fn exec(&self) -> io::Result<Output> {
        let mut command = self.build_command();

        let output = command.output()?;

        if output.status.success() {
            return Ok(output);
        } else {
            Err(Error::new(
                ErrorKind::Other,
                ProcessError {
                    msg: String::from("process did not exit successfully"),
                    args: self.get_args().to_vec(),
                    cwd: self
                        .get_cwd()
                        .map(|c| c.to_path_buf())
                        .unwrap_or(PathBuf::new()),
                    status: Some(output.status),
                    output: Some(output),
                },
            ))
        }
    }

    pub fn build_command(&self) -> Command {
        let mut command = Command::new(&self.program);
        if let Some(cwd) = self.get_cwd() {
            command.current_dir(cwd);
        }

        for arg in &self.args {
            command.arg(arg);
        }

        command
    }
}

// [TODO] All things errors.
#[derive(Debug)]
pub struct ProcessError {
    pub msg: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub status: Option<ExitStatus>,
    pub output: Option<Output>,
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "msg: {}\ncwd: {:?}\nargs: {:#?}\nstatus: {:#?}",
            &self.msg, &self.cwd, &self.args, &self.status
        )
    }
}

impl std::error::Error for ProcessError {
    fn description(&self) -> &str {
        "process error"
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::hofund;
    use crate::util::BifrostResult;

    #[test]
    fn test_basic_command_ls() -> BifrostResult<()> {
        let cwd = Path::new("tests")
            .join("test_user")
            .join("test_app_dir")
            .join("test_file_dir");

        let ls = ProcessBuilder {
            program: String::from("ls"),
            args: vec![],
            cwd: Some(cwd),
        };

        let output = ls.exec()?;

        let got = output.status.code().expect("expected 0i32");
        let expected = 0i32;
        assert_eq!(expected, got);

        let got = output.stdout;
        assert_eq!(b"file.txt\n", &got[..]);

        Ok(())
    }

    #[test]
    fn test_basic_command_cat() -> BifrostResult<()> {
        let cwd = Path::new("tests").join("test_user").join("test_app_dir");

        let cat = ProcessBuilder {
            program: String::from("cat"),
            args: vec![String::from("cat.txt")],
            cwd: Some(cwd),
        };

        let output = cat.exec()?;

        let got = output.status.code().expect("expected 0i32");
        let expected = 0i32;
        assert_eq!(expected, got);

        let got = output.stdout;
        assert_eq!(b"meow\n", &got[..]);

        Ok(())
    }

    #[test]
    fn test_basic_command_bad_cat() -> BifrostResult<()> {
        // [Fix Me] not a fan of this, but iterating along to see _what does_
        // need to occur here and how to structure/handle errors.
        use std::error::Error;

        let cwd = Path::new("tests").join("test_user").join("test_app_dir");

        let cat = ProcessBuilder {
            program: String::from("cat"),
            args: vec![String::from("bad_cat.txt")],
            cwd: Some(cwd),
        };

        if let Err(e) = cat.exec() {
            assert_eq!("process error", e.description());
        } else {
            assert!(cat.exec().is_err());
        }

        Ok(())
    }

    #[test]
    fn test_process_error_display() {
        let cwd = Path::new("tests").join("test_user").join("test_app_dir");

        let cat = ProcessBuilder {
            program: String::from("cat"),
            args: vec![String::from("bad_cat.txt")],
            cwd: Some(cwd),
        };

        assert!(cat.exec().is_err());
    }

    fn _test_with_rust_hello_world() -> BifrostResult<()> {
        use dirs;
        let home_path =
            dirs::home_dir().expect("could not get home path will testing `process_builder`");

        let process = ProcessBuilder {
            program: String::from("cargo"),
            args: vec![String::from("--version")],
            cwd: Some(home_path.clone()),
        };

        if let Ok(output) = process.exec() {
            if let Some(status_code) = output.status.code() {
                if status_code != 0i32 {
                    // cargo not installed...
                    // this test was just for fun anyways.
                    return Ok(());
                }
            }
        }

        let path = home_path.join(".bifrost").join("container").join("bifrost");

        let process = ProcessBuilder {
            program: String::from("cargo"),
            args: vec![String::from("new"), String::from("hello_world")],
            cwd: Some(path.clone()),
        };

        let output = process.exec()?;

        let got = output.status.code().expect("expected 0i32");
        let expected = 0i32;
        assert_eq!(expected, got);

        let path = path.clone().join("hello_world");

        let process = ProcessBuilder {
            program: String::from("cargo"),
            args: vec![String::from("run")],
            cwd: Some(path.clone()),
        };

        let output = process.exec()?;

        let got = output.status.code().expect("expected 0i32");
        let expected = 0i32;
        assert_eq!(expected, got);

        hofund::remove_dir_all(&path)?;

        Ok(())
    }
}
