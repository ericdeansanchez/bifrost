//! Primary structures, mehtods, and functions that facilitate `bifrost::ops`.
use crate::core::config::{self, CommandConfig, Config};
use crate::core::workingdir::WorkingDir;
use crate::util::{BifrostOptions, BifrostPath, BifrostResult, OperationInfo};

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;

use crate::ArgMatches;
use spinner::SpinnerBuilder;
use subprocess::{self, Popen, PopenConfig, Redirection};

/// Primary structure which `bifrost::ops operate upon.
///
/// A `WorkSpace` is composed of the following:
/// * name - the name of a Bifrost realm's `WorkSpace`.
/// * mode - describes _how_ `contents` of this `WorkSpace` should be handled.
/// * config - a [`Config`](struct.Config.html)
/// * contents - an `Option<Vec<WorkingDir>>`, see [`WorkingDir`](struct.WorkingDir.html)
/// * size - the size (in bytes) of all files contained within `WorkSpace`'s contents.
#[derive(Debug)]
pub struct WorkSpace {
    // Name of workspace that helps keep track of multiple `WorkSpace`'s.
    name: Option<String>,
    // Mode describing _how_ operations should be performed.
    mode: Mode,
    // Configuration information that describes the current Bifrost realm.
    config: Config,
    // Contents of the `WorkSpace` in the form of `WorkingDir`s.
    contents: Option<Vec<WorkingDir>>,
    // The size (in bytes) of all files contained within contents.
    size: u64,
}

/// A default `WorkSpace` is constructed with a default `Config` in `Mode::Normal`.
/// All other fields are `None`.
impl Default for WorkSpace {
    fn default() -> Self {
        WorkSpace {
            name: None,
            mode: Mode::Normal,
            config: Config::default(),
            contents: None,
            size: 0u64,
        }
    }
}

impl WorkSpace {
    /// Returns the name of a workspace.
    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    /// Returns the `Mode` of a workspace.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Returns the `Config` of a workspace.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Returns the contents of a workspace.
    pub fn contents(&self) -> Option<&Vec<WorkingDir>> {
        self.contents.as_ref()
    }

    /// Constructs a `LoadSpace`.
    pub fn to_load_space(config: Config, args: &ArgMatches) -> LoadSpace {
        WorkSpaceArgs::parse_load(config, &args).to_load_space()
    }

    /// Constructs a `ShowSpace`.
    pub fn to_show_space(config: Config, args: &ArgMatches) -> ShowSpace {
        WorkSpaceArgs::parse_show(config, &args).to_show_space()
    }

    /// Constructs an `UnloadSpace`.
    pub fn to_unload_space(config: Config, args: &ArgMatches) -> UnloadSpace {
        WorkSpaceArgs::parse_unload(config, args).to_unload_space()
    }

    /// Constructs a `RunSpace`.
    pub fn to_run_space(config: Config, args: &ArgMatches) -> RunSpace {
        WorkSpaceArgs::parse_run(config, &args).to_run_space()
    }
}

impl AsMut<WorkSpace> for WorkSpace {
    fn as_mut(&mut self) -> &mut WorkSpace {
        self
    }
}

/// The `Mode` specifiecs _how_ operations should be performed. A `WorkSpace`'s
/// contents will be handled differently based on this value (e.g. if `--auto` has
/// been specified then changes to source files will be detected and be automatically
/// re-`load`ed).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Mode {
    Auto,
    Modified,
    Normal,
}

/// This structure is similar to `WorkSpace`; however, there are a few differences.
/// Namely, `ignore_list` and `contents`. The `ignore_list` does not appear in `WorkSpace`
/// and `contents` is a optional `Vec<String>` instead of a `Vec<WorkingDir>`.
///
/// The reason for this is `WorkSpaceArgs` takes values from command line arguments
/// in the form of `ArgMatches` and from the `Config`. The value(s) of `contents` when
/// passed from the command line may not be appropriate values to construct `WorkingDir`s.
///
/// The `contents` are kept as strings until they have been processed to a point where
/// they are okay to use; processing may not be able to produce valid `contents`. In this
/// case, it is best to avoid constructing anything that is `BifrostOperable` or whose
/// owner is `BifrostOperable`.
///
/// New `WorkingDir`s take ownership of a copy of the `ignore_list` when they are constructed.
/// In previous iterations `WorkSpace`s owned an `ignore_list`. This way eliminates this one
/// small duplication.
struct WorkSpaceArgs {
    name: Option<String>,
    mode: Mode,
    config: Config,
    contents: Option<Vec<String>>,
    ignore_list: Vec<String>,
    opts: Option<BifrostOptions>,
}

impl WorkSpaceArgs {
    /// Constructs `WorkSpaceArgs` from  the mode, contents, name, and
    /// ignore_list from `ArgMatches` and a `Config`.
    fn parse_load(config: Config, args: &ArgMatches) -> Self {
        let ws_mode = WorkSpaceBuilder::get_mode(&args);
        let ws_name = WorkSpaceBuilder::get_name(&config);
        let ws_ignore_list = WorkSpaceBuilder::get_ignore_list(&config);
        let ws_contents = config::values_of("contents", &args).map_or(vec![], |v| v);

        WorkSpaceArgs {
            name: Some(ws_name),
            mode: ws_mode,
            config,
            contents: Some(ws_contents),
            ignore_list: ws_ignore_list,
            opts: None,
        }
    }

    /// Constructs `WorkSpaceArgs` from the mode, options, and current workspace name.
    fn parse_show(config: Config, args: &ArgMatches) -> Self {
        let ws_mode = WorkSpaceBuilder::get_mode(&args);
        let ws_opts = WorkSpaceBuilder::get_opts(&args);
        let ws_name = WorkSpaceBuilder::get_name(&config);
        WorkSpaceArgs {
            name: Some(ws_name),
            mode: ws_mode,
            config,
            contents: None,
            ignore_list: vec![],
            opts: ws_opts,
        }
    }

    /// Constructs `WorkSpaceArgs` from the current workspace name.
    fn parse_unload(config: Config, _args: &ArgMatches) -> Self {
        let ws_name = WorkSpaceBuilder::get_name(&config);
        WorkSpaceArgs {
            name: Some(ws_name),
            mode: Mode::Normal,
            config,
            contents: None,
            ignore_list: vec![],
            opts: Some(BifrostOptions::default()),
        }
    }

    /// Constructs `WorkSpaceArgs` from the workspace name and associated commands.
    fn parse_run(config: Config, _args: &ArgMatches) -> Self {
        let ws_name = WorkSpaceBuilder::get_name(&config);

        WorkSpaceArgs {
            name: Some(ws_name),
            mode: Mode::Normal,
            config,
            contents: None,
            ignore_list: vec![],
            opts: None,
        }
    }

    /// Constructs a `LoadSpace` for `WorkSpaceArgs`. `WorkSpaceArgs` has all the
    /// information needed to construct a `WorkSpace`. The _proposed_ `contents`
    /// are passed off to `WorkSpaceBuilder::args_to_paths` for some _basic_
    /// validation/manipulation.
    ///
    /// If no paths were returned, then there is nothing to construct `WorkingDir`s from
    /// and no reason to construct the `WorkSpace`.
    ///
    /// Otherwise, a new `WorkSpace` is constructed with new `contents` of type `Vec<WorkingDir>`
    /// and a `LoadSpace` is returned.
    fn to_load_space(self) -> LoadSpace {
        let mut working_dirs: Vec<WorkingDir> = Vec::new();
        if let Some(contents) = self.contents {
            if contents.is_empty() {
                working_dirs.push(WorkingDir::new(self.config.cwd()).ignore(&self.ignore_list));
            } else {
                let paths = WorkSpaceBuilder::args_to_paths(&self.config, contents, &is_loadable);
                if paths.is_empty() {
                    eprintln!(
                        "error: contents were passed but no arguments represent \
                         valid current working directory entries"
                    );
                    process::exit(1);
                }

                for path in paths {
                    working_dirs.push(WorkingDir::new(path).ignore(&self.ignore_list));
                }
            }
        }

        LoadSpace {
            workspace: WorkSpace {
                name: self.name,
                mode: self.mode,
                config: self.config,
                contents: Some(working_dirs),
                size: 0u64,
            },
            target: None,
        }
    }

    /// Translates `WorkSpaceArgs` into a `ShowSpace`.
    fn to_show_space(self) -> ShowSpace {
        ShowSpace {
            workspace: WorkSpace {
                name: self.name,
                mode: self.mode,
                config: self.config,
                contents: None,
                size: 0u64,
            },
            target: None,
            opts: self.opts,
        }
    }

    /// Translates `WorkSpaceArgs` into an `UnloadSpace`.
    fn to_unload_space(self) -> UnloadSpace {
        UnloadSpace {
            workspace: WorkSpace {
                name: self.name,
                mode: self.mode,
                config: self.config,
                contents: None,
                size: 0u64,
            },
            target: None,
        }
    }

    fn to_run_space(self) -> RunSpace {
        RunSpace {
            workspace: WorkSpace {
                name: self.name,
                mode: self.mode,
                config: self.config,
                contents: None,
                size: 0u64,
            },
            target: None,
            cmd: None,
        }
    }
}

/// Implementors of `BifrostOperable` are prep-able, build-able, and exec-utable.
/// These methods are chainable and are used to execute `bifrost::ops`; successful
/// execution returns a `BifrostResult` containing `OperationInfo` (or an error).
///
/// * `prep` - prepares the structure to be built
/// * `build` - builds the prepared structure
/// * `exec` - executes the structures primary function (e.g. `load` for `LoadSpace`, `show` for `ShowSpace`)
pub trait BifrostOperable {
    /// Prepares implementor to be built.
    fn prep(&mut self) -> BifrostResult<&mut BifrostOperable>;
    /// Builds implementor to be executed.
    fn build(&mut self) -> BifrostResult<&mut BifrostOperable>;
    /// Executes implementor's executive function(s).
    fn exec(&mut self) -> BifrostResult<OperationInfo>;
    /// Returns the operable `BifrostPath`.
    fn target(&self) -> Option<BifrostPath>;
}

/// Primary data structure used to `load` `WorkSpace`s into the Bifrost container.
#[derive(Debug)]
pub struct LoadSpace {
    /// The `WorkSpace` to be loaded.
    workspace: WorkSpace,
    /// The target path to `load` the `WorkSpace` to.
    target: Option<BifrostPath>,
}

/// A `LoadSpace`'s primary goal is to `load` contents into the Bifrost container.
/// However, it also implements helper methods that make accessing its nested
/// structure a little more ergonomic at this level.
impl LoadSpace {
    /// Returns a reference to the underlying `WorkSpace`.
    pub fn workspace(&self) -> &WorkSpace {
        &self.workspace
    }

    /// Returns a reference to the underlying `home_path` `PathBuf` defined
    /// upon configuration.
    pub fn home_path(&self) -> &PathBuf {
        self.workspace.config().home_path()
    }

    /// Returns a optional reference to the underlying `WorkSpace` name.
    pub fn name(&self) -> Option<&String> {
        self.workspace.name()
    }

    /// Returns the underlying `WorkSpace`'s contents `as_mut`.
    pub fn contents_as_mut(&mut self) -> Option<&mut Vec<WorkingDir>> {
        self.workspace.contents.as_mut()
    }

    /// Returns the underlying `WorkSpace` `as_mut`.
    pub fn workspace_as_mut(&mut self) -> &mut WorkSpace {
        self.workspace.as_mut()
    }

    /// Loads this spaces's `WorkSpace` to the target `BifrostPath`.
    ///
    /// # Panics
    ///
    /// * if the call to [`prep`] has failed or
    /// * if it cannot unwrap a `WorkSpace::name`
    ///
    /// It is possible that this method **panics**, however, this means
    /// that this method was called prior to a call to [`prep`] or that
    /// the call to [`prep`] failed.
    ///
    /// It might be possible for this method to attempt to unwrap a `WorkSpace::name`
    /// that is `None`. This seems **unlikely**, however, this can be ruled out if
    /// it can be shown that `WorkSpace::default` is the only operation that
    /// produces a `WorkSpace` with a name that is `None`.
    pub fn load(&mut self) -> BifrostResult<OperationInfo> {
        let path = self
            .target
            .clone()
            .expect("BUG: `BifrostPath` should not be `None` here");

        let mut nbytes = 0u64;

        // The contents must be mutable because loading a `WorkingDir`
        // mutates its underlying `dirs` field.
        if let Some(contents) = self.contents_as_mut() {
            for wd in contents {
                let info = wd.load(path.clone())?;
                if let Some(n) = info.bytes {
                    nbytes += n;
                }
            }
        }

        // If the number of bytes loaded does not equal the number of bytes
        // recorded when the workspace was created, then the workspace has not
        // been properly loaded.
        if nbytes != self.workspace.size {
            failure::bail!("error: could not `load` all contents");
        }

        // The name really should not be `None` at this point.
        let name = self
            .name()
            .expect("BUG: `LoadSpace::load` failed to unwrap `WorkSpace::name`")
            .clone();

        // Returns the sum of bytes written and the name of the workspace they
        // were written from.
        Ok(OperationInfo {
            bytes: Some(nbytes),
            name,
            text: None,
            ..Default::default()
        })
    }
}

/// Implements `BifrostOperable` for `LoadSpace`.
/// A `LoadSpace` is `prep`-able, `build`-able, and `exec`-utable. These methods
/// follow a builder-esq pattern. They are meant to be called in sequence (chained).
/// Both `prep` and `build` return `BifrostResult`s containing structures that implement
/// `BifrostOperable` (or errors). Conversely, `build` is a consuming operation that
/// returns a `BifrostResult` containing the `OperationInfo`.
///
/// # Examples
///
/// ```compile_fail
/// // Both `prep` and `build` return `BifrostResult`s containing structures that
/// // implement `BifrostOperable`.
/// let bifrost_operable = WorkSpace::to_load_space(config, &args)
///        .prep()?
///        .build()?;
///
/// // Conversely, `exec` is a consuming operation and returns a `BifrostResult`
/// // containing the `OperationInfo` (i.e. number of bytes written).
/// let operation_info = WorkSpace::to_load_space(config, &args)
///        .prep()?
///        .build()?
///        .exec()?;
/// ```
impl BifrostOperable for LoadSpace {
    /// Prepares a `LoadSpace` by ensuring that its target destination (`BifrostPath`)
    /// does not already exist. Reassigns `self.bifrost_path` if and only if the
    /// `BifrostPath` is unique.
    fn prep(&mut self) -> BifrostResult<&mut BifrostOperable> {
        if self.target.is_some() {
            failure::bail!("error: a `BifrostPath` has already been prepared for this `LoadSpace`");
        }
        self.target = Some(BifrostPath::new(self.home_path(), self.name())?);
        Ok(self)
    }

    /// Builds a `LoadSpace` by calling `walk` on each underlying `WorkingDir` owned
    /// by the workspace's contents. Reassigns `self.workspace.contents` and reassigns
    /// `self.workspace.size` if and only if each `walk` was successful.
    fn build(&mut self) -> BifrostResult<&mut BifrostOperable> {
        if let Some(contents) = self.workspace.contents.as_mut() {
            let mut nybtes = 0u64;
            let mut walked_content: Vec<WorkingDir> = vec![];
            while let Some(entry) = contents.pop() {
                let wd = entry.walk()?;
                nybtes += wd.size();
                walked_content.push(wd);
            }
            // Update the `WorkSpace::size` now that directories have been walked.
            self.workspace.size = nybtes;
            self.workspace.contents = Some(walked_content)
        }
        Ok(self)
    }

    /// Executes a `LoadSpace`'s primary function: `load`.
    fn exec(&mut self) -> BifrostResult<OperationInfo> {
        Ok(self.load()?)
    }

    /// Returns a `clone`d version of the target `BifrostPath` (or `None`).
    fn target(&self) -> Option<BifrostPath> {
        if let Some(ref path) = self.target {
            return Some(path.clone());
        }
        None
    }
}

/// Primary data structure used to `show` `WorkSpace`s that exist within the
/// Bifrost container.
#[derive(Debug)]
pub struct ShowSpace {
    /// The `WorkSpace` to be loaded.
    workspace: WorkSpace,
    /// The target path to `load` the `WorkSpace` to.
    target: Option<BifrostPath>,
    /// Options that control _how_ a `ShowSpace` is displayed.
    opts: Option<BifrostOptions>,
}

/// A `ShowSpace`'s primary goal is to `show` the contents in the Bifrost container
/// that have been `load`ed from the current workspace.
impl ShowSpace {
    /// Returns a reference to the underlying `home_path` `PathBuf` defined
    /// upon configuration.
    pub fn home_path(&self) -> &PathBuf {
        self.workspace.config().home_path()
    }

    /// Returns a optional reference to the underlying `WorkSpace` name.
    pub fn name(&self) -> Option<&String> {
        self.workspace.name()
    }

    /// Returns the result of the appropriate method call given `BifrostOptions`.
    /// The possible methods are as follows:
    /// * `show_default` - displays the minimal amount of information about the
    /// current bifrost realm.
    /// * `show_all` - displays as much information as is available about the
    /// current bifrost realm.
    /// * `show_diff` - displays only files that have been modified in the current
    /// bifrost realm but have not be re-loaded into the bifrost container realm.
    pub fn show(&self) -> BifrostResult<OperationInfo> {
        if let Some(ref opts) = self.opts {
            if opts.verbose {
                return Ok(self.show_all()?);
            }
        }
        return Ok(self.show_default()?);
    }

    fn show_default(&self) -> BifrostResult<OperationInfo> {
        let mut op_info = OperationInfo::new();

        if let Ok(current_dir) = env::current_dir() {
            if let Some(ref target) = self.target {
                env::set_current_dir(&target.path)?;
                let out = process::Command::new("ls")
                    .arg("-lr")
                    .output()
                    .unwrap()
                    .stdout;
                op_info.text = Some(out);
            }
            env::set_current_dir(current_dir)?;
        }

        op_info.name = self
            .name()
            .expect("BUG: `ShowSpace::show_default` expected `self.name` to be `Some`")
            .to_string();

        Ok(op_info)
    }

    fn show_all(&self) -> BifrostResult<OperationInfo> {
        let mut op_info = OperationInfo::new();

        if let Ok(current_dir) = env::current_dir() {
            if let Some(ref target) = self.target {
                env::set_current_dir(&target.path)?;
                let out = process::Command::new("ls")
                    .arg("-laR")
                    .output()
                    .unwrap()
                    .stdout;
                op_info.text = Some(out);
            }
            env::set_current_dir(current_dir)?;
        }

        op_info.name = self
            .name()
            .expect("BUG: `ShowSpace::show_all` expected `self.name` to be `Some`")
            .to_string();

        Ok(op_info)
    }
}

/// Implements `BifrostOperable` for `ShowSpace`.
/// A `ShowSpace` is `prep`-able, `build`-able, and `exec`-utable.
impl BifrostOperable for ShowSpace {
    /// Prepares a `ShowSpace` by setting its `BifrostPath` from the current
    /// existing workspace.
    fn prep(&mut self) -> BifrostResult<&mut BifrostOperable> {
        let path = BifrostPath::try_from_existing(self.home_path(), self.name())?;
        self.target = Some(path);
        Ok(self)
    }

    /// Builds a `ShowSpace` given its `BifrostOptions`.
    fn build(&mut self) -> BifrostResult<&mut BifrostOperable> {
        if let Some(ref opts) = self.opts {
            if opts.verbose {
                return Ok(self);
            } else if opts.diff {
                // [TODO] build the space for a --diff display
                println!("building for --diff (gotta write that diff-er now...)");
            }
        }
        Ok(self)
    }

    /// Executes a `ShowSpace`'s primary function: `show`.
    fn exec(&mut self) -> BifrostResult<OperationInfo> {
        Ok(self.show()?)
    }

    /// Returns a `clone`d version of the target `BifrostPath`.
    fn target(&self) -> Option<BifrostPath> {
        if let Some(ref path) = self.target {
            return Some(path.clone());
        }
        None
    }
}

/// Primary data structure used to `unload` a `WorkSpace` that exist within the
/// Bifrost container.
#[derive(Debug)]
pub struct UnloadSpace {
    /// The `WorkSpace` to be loaded.
    workspace: WorkSpace,
    /// The target path to `load` the `WorkSpace` to.
    target: Option<BifrostPath>,
}

/// An `UnloadSpace`'s primary goal is to `unload` the contents from the Bifrost container
/// that have been `load`ed from the current workspace.
impl UnloadSpace {
    /// Returns a reference to the underlying `home_path` `PathBuf` defined
    /// upon configuration.
    pub fn home_path(&self) -> &PathBuf {
        self.workspace.config().home_path()
    }

    /// Returns a optional reference to the underlying `WorkSpace` name.
    pub fn name(&self) -> Option<&String> {
        self.workspace.name()
    }

    /// Unloads the current workspace from the Bifrost container by removing
    /// the entire directory its `BifrostPath` denotes.
    ///
    /// If the `BifrostPath` is `Some`, then there is an attempt to remove the workspace.
    /// If this attempt fails, then a message is written to stdout before exiting.
    pub fn unload(&self) -> BifrostResult<OperationInfo> {
        use crate::core::hofund;

        if let Some(ref target) = self.target {
            match hofund::remove_dir_all(&target.path) {
                Ok(_) => {
                    return Ok(OperationInfo {
                        name: self
                            .name()
                            .expect("BUG: `UnloadSpace::unload` expected `name` to be `Some`")
                            .to_string(),
                        ..Default::default()
                    })
                }
                Err(e) => {
                    io::stdout().write_fmt(format_args!(
                        "failed: something went wrong while attempting to `unload` `{}` due to {}",
                        target.path.to_str().unwrap(),
                        e
                    ))?;
                    process::exit(1);
                }
            }
        } else {
            io::stdout().write_fmt(format_args!(
                "`BifrostPath` was found to be `None` while attempting to `Unload`"
            ))?;
            process::exit(1);
        }
    }
}

/// Implements `BifrostOperable` for `UnloadSpace`.
/// An `UnloadSpace` is `prep`-able, `build`-able, and `exec`-utable.
impl BifrostOperable for UnloadSpace {
    /// Prepares an `UnloadSpace` by setting its `BifrostPath` from the current
    /// existing workspace.
    fn prep(&mut self) -> BifrostResult<&mut BifrostOperable> {
        let path = BifrostPath::try_from_existing(self.home_path(), self.name())?;
        self.target = Some(path);
        Ok(self)
    }

    /// Builds an `UnloadSpace`.
    fn build(&mut self) -> BifrostResult<&mut BifrostOperable> {
        Ok(self)
    }

    /// Executes an `UnloadSpace`'s primary function: `unload`.
    fn exec(&mut self) -> BifrostResult<OperationInfo> {
        Ok(self.unload()?)
    }

    /// Returns a cloned version of the target `BifrostPath` (or None).
    fn target(&self) -> Option<BifrostPath> {
        if let Some(ref path) = self.target {
            return Some(path.clone());
        }
        None
    }
}

/// Primary data structure used to `run` `WorkSpace`s that exist within the
/// Bifrost container.
#[derive(Debug)]
pub struct RunSpace {
    /// The `WorkSpace`.
    workspace: WorkSpace,
    /// The target path.
    target: Option<BifrostPath>,
    /// The commands to be executed within the bifrost container.
    cmd: Option<CommandConfig>,
}

/// An `RunSpace`'s primary goal is to `run` commands.
impl RunSpace {
    /// Returns a reference to the underlying `home_path` `PathBuf` defined
    /// upon configuration.
    pub fn home_path(&self) -> &PathBuf {
        self.workspace.config().home_path()
    }

    /// Returns a optional reference to the underlying `WorkSpace` name.
    pub fn name(&self) -> Option<&String> {
        self.workspace.name()
    }

    pub fn run(&self) -> BifrostResult<OperationInfo> {
        let home_path = match dirs::home_dir() {
            Some(path) => path,
            _ => {
                io::stderr().write("error: failed to `run` target is `None`".as_bytes())?;
                process::exit(1);
            }
        };

        let path = home_path
            .join(".bifrost")
            .join("container:")
            .join("bifrost");

        let path = path
            .to_str()
            .expect("error: could not unwrap path to `run`");

        let target_dir = self
            .name()
            .expect("error: `run` expected name to be `Some`");

        let mut output = match self.cmd {
            Some(ref b) => RunSpace::_run(b, &path, &target_dir)?,
            None => failure::bail!("error: failed to `run` `CommandConfig` is `None`"),
        };

        output.name = self
            .name()
            .unwrap_or(&String::from("run-default"))
            .to_owned();

        Ok(output)
    }

    fn _run(cmd: &CommandConfig, path: &str, target_dir: &str) -> BifrostResult<OperationInfo> {
        match cmd.get_cmds() {
            Some(c) => Ok(RunSpace::_run_process(c, path, target_dir)?),
            _ => failure::bail!("bail for now  could not get cmds from command config... [FIX]"),
        }
    }

    fn _run_process(
        cmds: &Vec<String>,
        path: &str,
        target_dir: &str,
    ) -> BifrostResult<OperationInfo> {
        let mut process = Popen::create(
            &[
                "docker",
                "run",
                "--rm",
                "-i",
                "--volume",
                path,
                "bifrost:0.1",
            ],
            PopenConfig {
                stdout: Redirection::Pipe,
                stdin: Redirection::Pipe,
                stderr: Redirection::Pipe,
                ..Default::default()
            },
        )
        .expect("Popen failure");

        // [FIX ME] - this works right now, however, it is
        // ugly and brittle.
        let md_cmd = cmds.join(" && ");
        let mounted_cmd = format!(
            "bash -c \"cd /bifrost/bifrost; cd {}; {}; \"",
            target_dir, md_cmd
        );

        let _sp = SpinnerBuilder::new("Running...".into())
            .spinner(vec![
                "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏",
            ])
            .start();

        match process
            .communicate(Some(&mounted_cmd))
            .expect("communicate failure")
        {
            (Some(stdout), Some(stderr)) => Ok(OperationInfo {
                stdout,
                stderr,
                ..Default::default()
            }),
            (Some(stdout), _) => Ok(OperationInfo {
                stdout,
                ..Default::default()
            }),
            (_, Some(stderr)) => Ok(OperationInfo {
                stderr,
                ..Default::default()
            }),
            (_, _) => Ok(OperationInfo {
                // [FIX]
                ..Default::default()
            }),
        }
    }
}

/// Implements `BifrostOperable` for `RunSpace`.
/// A `RunSpace` is `prep`-able, `build`-able, and `exec`-utable.
impl BifrostOperable for RunSpace {
    fn prep(&mut self) -> BifrostResult<&mut BifrostOperable> {
        // Path should exist
        let path = BifrostPath::try_from_existing(self.home_path(), self.name())?;
        self.target = Some(path);

        // Take the `command` from the `config`.
        self.cmd = WorkSpaceBuilder::take_command_config(&mut self.workspace.config);

        Ok(self)
    }

    /// Builds an `RunSpace`.
    fn build(&mut self) -> BifrostResult<&mut BifrostOperable> {
        Ok(self)
    }

    /// Executes a `RunSpace`'s primary function: `run`.
    fn exec(&mut self) -> BifrostResult<OperationInfo> {
        Ok(self.run()?)
    }

    /// Returns a cloned version of the target `BifrostPath` (or None).
    fn target(&self) -> Option<BifrostPath> {
        if let Some(ref path) = self.target {
            return Some(path.clone());
        }
        None
    }
}

/// `WorkSpaceBuilder` provides utility functions needed to construct `WorkSpaceArgs`.
/// It is a no-field struct used to signal a division of labor/responsibility.
pub struct WorkSpaceBuilder;

impl WorkSpaceBuilder {
    // Gets the ignore list from the manifest if it exists. Otherwise, an
    // empty vector will be returned meaning that no files will be ignored in
    // the `WorkSpace`.
    fn get_ignore_list(config: &Config) -> Vec<String> {
        match config
            .manifest()
            .and_then(|m| m.get_workspace_config().and_then(|ws| ws.ignore()))
        {
            Some(list) => list.to_owned(),
            None => Vec::new(),
        }
    }

    // Gets the name of the workspace if it exists; otherwise, the workspace
    // name is derived from the current working directory's top-level directory.
    fn get_name(config: &Config) -> String {
        const DEFAULT_TOML_NAME: &str = "workspace name";

        fn name_from_cwd(cwd: &PathBuf) -> String {
            match cwd.file_name() {
                Some(name) => String::from(
                    name.to_str()
                        .expect("error: could not `unwrap` name from cwd"),
                ),
                None => panic!("error: could not determine name from cwd from `file_name`"),
            }
        }

        match config
            .manifest()
            .and_then(|m| m.get_workspace_config().and_then(|ws| ws.name()))
        {
            None => name_from_cwd(config.cwd()),
            Some(name) => {
                if name == DEFAULT_TOML_NAME {
                    return name_from_cwd(config.cwd());
                } else {
                    return String::from(name);
                }
            }
        }
    }

    /// Gets the `BifrostOptions` gathered from `ArgMatches` if they are present and
    /// returns `None` if they are not present.
    fn get_opts(args: &ArgMatches) -> Option<BifrostOptions> {
        if args.args.is_empty() {
            return None;
        }

        if args.is_present("all") {
            return Some(BifrostOptions {
                verbose: true,
                ..Default::default()
            });
        } else if args.is_present("diff") {
            return Some(BifrostOptions {
                diff: true,
                ..Default::default()
            });
        }
        return None;
    }

    /// `take`'s the `command_config` from the `Config`.
    fn take_command_config(config: &mut Config) -> Option<CommandConfig> {
        config.manifest_mut().and_then(|m| m.take_command_config())
    }

    // Returns the `Mode` flag.
    fn get_mode(args: &ArgMatches) -> Mode {
        if args.is_present("auto") {
            return Mode::Auto;
        } else if args.is_present("modified") {
            return Mode::Modified;
        }
        Mode::Normal
    }

    // Convenience function that `strip`s trailing slashes from command line arguments,
    // `check`s if the current working directory has paths that contain these arguments,
    // and then returns absolute-path-versions of these arguments.
    // More validation may need to be done; this is a naive implementation.
    fn args_to_paths(
        config: &Config,
        args: Vec<String>,
        f: &Fn(&Config, &str) -> BifrostResult<bool>,
    ) -> Vec<PathBuf> {
        let ws_path_names = strip(&args);
        let ws_paths = check(&config, ws_path_names, f);
        return to_abs_paths(config.cwd(), ws_paths);
    }
}

// Collects `names` that can be considered valid paths for a given operation. If
// the `BifrostResult` returned by `f` cannot be unwrapped then the value an `Ok`
// result would have contained is considered to be false.
fn check(
    config: &Config,
    names: Vec<String>,
    f: &Fn(&Config, &str) -> BifrostResult<bool>,
) -> Vec<String> {
    names
        .into_iter()
        .filter(|n| f(&config, n).unwrap_or(false))
        .collect()
}

// A `name` is valid if it exists within the current Bifrost realm.
fn is_loadable(config: &Config, name: &str) -> BifrostResult<bool> {
    // Home or root paths are not loadable.
    if config.home_path().as_path() == Path::new(name) {
        return Ok(false);
    }

    // A path that already contains .bifrost or container is not loadable.
    if name.contains(".biforst") || name.contains("container") {
        return Ok(false);
    }

    // A `name` must exist within the current working directory in order for
    // it to be loadable.
    for entry in fs::read_dir(config.cwd())? {
        let entry = entry?;
        if entry.file_name() == name {
            return Ok(true);
        }
    }
    // No other cases are loadable.
    Ok(false)
}

// Returns `names` without trailing slashes.
fn strip(names: &Vec<String>) -> Vec<String> {
    let mut names_without_trailing_slashes: Vec<String> = Vec::new();
    for n in names {
        names_without_trailing_slashes.push(String::from(strip_trailing_slash(n)));
    }
    names_without_trailing_slashes
}

// Returns a `name` without any trailing slashes.
fn strip_trailing_slash(name: &str) -> &str {
    if name.len() > 1 && (name.ends_with("\\") || name.ends_with("/")) {
        return &name[..name.len() - 1];
    }
    name
}

// Converts `name`d strings to `PathBuf`s if and only if the name is contained
// within a `PathBuf` that resides in the current working directory.
fn to_abs_paths<P>(cwd: P, paths: Vec<String>) -> Vec<PathBuf>
where
    P: AsRef<Path>,
{
    let mut abs_paths: Vec<PathBuf> = Vec::new();
    if paths.is_empty() {
        return abs_paths;
    }

    if let Ok(read_dir) = fs::read_dir(cwd) {
        for entry in read_dir {
            let entry =
                entry.expect("BUG: `BifrostPath::check` passed but `to_abs_paths` has failed");
            for name in paths.iter() {
                if entry.file_name() == name[..] {
                    abs_paths.push(entry.path())
                }
            }
        }
    }
    abs_paths
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default_workspace() {
        let ws = WorkSpace::default();
        let msg = "BUG: `WorkSpace::default` succeeds but `test_default_workspace` fails to";
        let home_path = dirs::home_dir().expect(&format!("{} {}", msg, "unwrap `dirs::home_dir`"));
        let cwd = std::env::current_dir()
            .expect(&format!("{} {}", msg, "unwrap `std::env::current_dir`"));

        assert_eq!(home_path, *ws.config.home_path());
        assert_eq!(cwd, *ws.config.cwd());
        assert_eq!(Mode::Normal, ws.mode);
        assert!(ws.name.is_none() && ws.contents.is_none());
    }

    #[test]
    fn test_to_abs_paths() {
        let names: Vec<String> = vec![
            String::from("file.txt"),
            String::from("file.cpp"),
            String::from("test_dir_nested"),
            String::from("test_ignore_file.txt"),
            String::from("file.c"),
            String::from("file.rs"),
            // Not valid
            String::from("random"),
            // Not valid
            String::from("not in dir"),
        ];

        const LEFT: usize = 6;
        let abs_paths = to_abs_paths("tests/test_dir", names);
        assert_eq!(LEFT, abs_paths.len());

        let abs_paths = to_abs_paths("tests/test_dir", vec![]);
        assert!(abs_paths.is_empty());
    }

    #[test]
    fn test_strip_trailing_slash() {
        let n0 = "";
        assert_eq!(n0, strip_trailing_slash(n0));

        let n1 = "/";
        assert_eq!(n1, strip_trailing_slash(n1));

        let n2 = "src/";
        assert_eq!("src", strip_trailing_slash(n2));

        let n3 = r#"src\"#;
        assert_eq!("src", strip_trailing_slash(n3));
    }

    #[test]
    fn test_strip() {
        let left: Vec<String> = vec![
            String::from(""),
            String::from("/"),
            String::from("src/"),
            String::from(r#"src\"#),
        ];

        let right = strip(&left);
        assert_eq!(left.len(), right.len());
    }

    // figure out how to run this test
    fn _test_is_loadable() -> BifrostResult<()> {
        let current_dir = std::env::current_dir()?;
        let test_path = Path::new("tests").join("test_dir");
        std::env::set_current_dir(test_path)?;
        let config = Config::default();
        let mut entries: Vec<String> = vec![];
        for e in fs::read_dir(config.cwd())? {
            let e = e?;
            let fname = e.file_name();
            let s = fname
                .into_string()
                .expect("error: test_is_loadable failed to convert `file_name` `into_string`");
            entries.push(s);
        }

        for e in entries {
            assert!(is_loadable(&config, &e)?);
        }
        std::env::set_current_dir(current_dir)?;
        Ok(())
    }
}
