//! # Generates the top-level command line application.
//! Generates a `clap::App` application. As the overall program grows, it
//! should be easy to plug-in new commands and functionality without having to
//! do much in `bifrost::app::app()`. I want to promote modularity without
//! overly constraining ourselves or introducing too much abstraction and/or
//! indirection unnecessarily.
//!
//! The process of building up the subcommands and the arguments to subcommands
//! is a little repetitive right now. However, this is somewhat of a good thing
//! as it demonstrates a _pattern_ of how to implement and extend our feature
//! set; a pattern that _seeks_ to be as modular and as intuitive as possible.
use clap::SubCommand;

use crate::util::template::APP_TEMPLATE;
use crate::util::template::BIFROST_USAGE;
use crate::util::template::SUBCOMMAND_HELP_TEMPLATE;

type Arg = clap::Arg<'static, 'static>;
type App = clap::App<'static, 'static>;

/// Builds a basic `clap::App` object and leaves the heavy
/// lifting to `all_sub_commands`.
///
/// All supported subcommands are generated with a call to
/// `all_sub_commands` which returns a `Vec<App>`. Each
/// subcommand can support arguments of its own.
///
/// The arguments are built up in the same way: a function to
/// build the `Arg::with_name` and push it onto a `Vec<Arg>`
/// and a function to essentially register each argument.
pub fn cli() -> App {
    let mut app = App::new("bifrost")
        .about("Bridging the tool gap")
        .version("0.1")
        .author(crate_authors!(",\n"))
        .template(APP_TEMPLATE)
        .usage(BIFROST_USAGE);

    app = app.subcommands(all_sub_commands());
    app
}

/// Generates a sequence of all subcommands this app supports.
/// Each subcommand has a function associated with that builds
/// the command and pushes it onto the `sub_command` vector.
/// `bifrost::app` uses this vector to construct its subcommand
/// list.
fn all_sub_commands() -> Vec<App> {
    let mut sub_commands: Vec<App> = vec![];
    sub_command_init(&mut sub_commands);
    sub_command_run(&mut sub_commands);
    sub_command_show(&mut sub_commands);
    sub_command_unload(&mut sub_commands);
    sub_command_load(&mut sub_commands);

    sub_commands
}

fn sub_command_run(commands: &mut Vec<App>) {
    const ABOUT: &str = "Run command string(s) on a bifrost workspace";
    const USAGE: &str = "bifrost run [OPTIONS]";

    let s = SubCommand::with_name("run").about(ABOUT).usage(USAGE);

    commands.push(s);
}

fn sub_command_show(commands: &mut Vec<App>) {
    const ABOUT: &str = "Display files currently in the bifrost container";
    const USAGE: &str = "bifrost show [OPTIONS]";

    let s = SubCommand::with_name("show").about(ABOUT).usage(USAGE);

    commands.push(s);
}

fn sub_command_unload(commands: &mut Vec<App>) {
    const ABOUT: &str = "Unload a workspace from the bifrost container";
    const USAGE: &str = "bifrost unload [OPTIONS]";

    let s = SubCommand::with_name("unload").about(ABOUT).usage(USAGE);

    commands.push(s);
}

fn sub_command_load(commands: &mut Vec<App>) {
    const SHORT: &str = "Load directory, file, or files into the bifrost container";
    const USAGE: &str = "
    bifrost load [OPTIONS] <contents>
    bifrost load [OPTIONS] project/
    bifrost load [OPTIONS] main.c avl.c bst.c
";

    let mut s = SubCommand::with_name("load")
        .about(SHORT)
        .template(SUBCOMMAND_HELP_TEMPLATE)
        .usage(USAGE);

    for a in all_load_args() {
        s = s.arg(a);
    }

    commands.push(s);
}

/// Generates a sequence of all args that `load` supports.
/// Each subcommand is a `clap::App` under the hood that
/// supports its own arguments. This function builds
/// up the each of the `clap::App`'s arguments.
fn all_load_args() -> Vec<Arg> {
    let mut load_args: Vec<Arg> = vec![];
    arg_load_auto(&mut load_args);
    arg_load_contents(&mut load_args);
    arg_load_modified(&mut load_args);

    load_args
}

fn arg_load_auto(args: &mut Vec<Arg>) {
    const SHORT: &str = "Turn on auto reloading";
    const LONG: &str = "When `auto` is enabled, modified files are automatically \
detected and loaded into the bifrost container without having to manually run \
`bifrost load --modified <files>`:

\t$ bifrost run


";

    let a = Arg::with_name("auto")
        .multiple(true)
        .long("auto")
        .short("a")
        .help(SHORT)
        .long_help(LONG);

    args.push(a);
}

/// Builds `args`. Running `bifrost load`, without specifying what should be
/// loaded, will result in the entire current working directory to be loaded
/// into the container.
fn arg_load_contents(args: &mut Vec<Arg>) {
    const LONG: &str = "Directory, files, or singular file to load into the \
                        bifrost container";

    let a = Arg::with_name("contents")
        .multiple(true)
        .help(LONG)
        .long_help(LONG);

    args.push(a);
}

fn arg_load_modified(args: &mut Vec<Arg>) {
    const SHORT: &str = "Load only files that have been \
                         modified since the last run";
    const LONG: &str = "
Load only files that have been modified since the last
run.


";

    let a = Arg::with_name("modified")
        .multiple(true)
        .long("modified")
        .short("m")
        .help(SHORT)
        .long_help(LONG);

    args.push(a);
}

fn sub_command_init(commands: &mut Vec<App>) {
    const ABOUT: &str = "Initialize a bifrost directory within the current working directory";
    const USAGE: &str = "bifrost init
    bifrost init --project=jupiter
    bifrost init --project=quasar --workspace=shattuck --ignore=relu
    bifrost init --project=jupiter --ignore .git --container=docker";

    let mut s = SubCommand::with_name("init")
        .about(ABOUT)
        .template(SUBCOMMAND_HELP_TEMPLATE)
        .usage(USAGE);

    for a in all_init_args() {
        s = s.arg(a);
    }

    commands.push(s);
}

fn all_init_args() -> Vec<Arg> {
    let mut init_args: Vec<Arg> = vec![];
    arg_init_project(&mut init_args);
    arg_init_workspace(&mut init_args);
    arg_init_ignore(&mut init_args);
    arg_init_command(&mut init_args);
    arg_init_container(&mut init_args);
    init_args
}

fn arg_init_project(args: &mut Vec<Arg>) {
    const SHORT: &str = "The project name for a Bifrost realm";
    const LONG: &str = "
The project name for a Birfrost realm. This is the
name will appear in the Bifrost.toml manifest.

    bifrost init --project=<name of project>
    bifrost init --project=asgard
    bifrost init --project=midgard


";

    let a = Arg::with_name("project")
        .help(SHORT)
        .long_help(LONG)
        .short("p")
        .long("project")
        .takes_value(true)
        .require_equals(true);

    args.push(a);
}

fn arg_init_workspace(args: &mut Vec<Arg>) {
    const SHORT: &str = "The workspace name for a Bifrost realm";
    const LONG: &str = "
The workspacce name for a Birfrost realm. This is the name will appear
in the Bifrost.toml manifest.

    bifrost init --workspace=<name of workspace>


";

    let a = Arg::with_name("workspace")
        .help(SHORT)
        .long_help(LONG)
        .short("w")
        .long("workspace")
        .takes_value(true)
        .require_equals(true);

    args.push(a);
}

fn arg_init_ignore(args: &mut Vec<Arg>) {
    const SHORT: &str = "The files and/or directories to be ignored";
    const LONG: &str = "
The names of files and/or directories to be ignored from the Bifrost
workspace. Typically, these values are .git directories, .gitignore
files, or other contents that are non-essential to the functionality
of a project.

    bifrost init --ignore .git
    bifrost init --ignore .git .gitignore
    bifrost init --ignore .gitignore target


";

    let a = Arg::with_name("ignore")
        .help(SHORT)
        .long_help(LONG)
        .short("i")
        .long("ignore")
        .multiple(true)
        .takes_value(true);

    args.push(a);
}

fn arg_init_command(args: &mut Vec<Arg>) {
    const SHORT: &str = "The command string(s) to be passed to the Bifrost container realm";
    const LONG: &str = "
The command string(s) will be passed to the Bifrost container realm.
From there, they will be passed to the specified application to be
run within the context of the container. An ignore-list can also be
specified in the Bifrost.toml manifest.

    bifrost init --command <command string and/or args>...
    bifrost init --command ls
    bifrost init --command bifrost load src/


";

    let a = Arg::with_name("command")
        .help(SHORT)
        .long_help(LONG)
        .short("c")
        .long("command")
        .takes_value(true)
        .multiple(true);

    args.push(a);
}

fn arg_init_container(args: &mut Vec<Arg>) {
    const SHORT: &str = "The name of the container to be utilized within the Bifrost realm";
    const LONG: &str = "The name of the container to be utilized within the
Bifrost realm. 

    bifrost init --container=<name of container>
    bifrost init --container=docker


";

    let a = Arg::with_name("container")
        .help(SHORT)
        .long_help(LONG)
        .short("t")
        .long("container")
        .takes_value(true)
        .require_equals(true);

    args.push(a);
}
