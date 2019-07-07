//! # Generates the top-level command line application.
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
    sub_command_setup(&mut sub_commands);
    sub_command_init(&mut sub_commands);
    sub_command_run(&mut sub_commands);
    sub_command_show(&mut sub_commands);
    sub_command_unload(&mut sub_commands);
    sub_command_load(&mut sub_commands);

    sub_commands
}

fn sub_command_setup(commands: &mut Vec<App>) {
    const ABOUT: &str = "Setup the utilities bifrost requires to operate";
    const USAGE: &str = "bifrost setup";
    const LONG: &str = "
 Setup the utilities bifrost requires to operate. This command is intended
 to be run once after a successful install. Bifrost aims to be transparent
 about changes it makes to a system. This command creates the following
 directory structure in your $HOME directory:

.bifrost
└── container
    └── bifrost
        └── Dockerfile
 
 ";

    let s = SubCommand::with_name("setup")
        .about(ABOUT)
        .long_about(LONG)
        .usage(USAGE);

    commands.push(s);
}

fn sub_command_unload(commands: &mut Vec<App>) {
    const ABOUT: &str = "Unload a workspace from the bifrost container";
    const USAGE: &str = "bifrost unload [OPTIONS]";

    let s = SubCommand::with_name("unload").about(ABOUT).usage(USAGE);

    commands.push(s);
}

fn sub_command_run(commands: &mut Vec<App>) {
    const ABOUT: &str = "Run command string(s) on a bifrost workspace";
    const USAGE: &str = "bifrost run [OPTIONS]";

    let mut s = SubCommand::with_name("run").about(ABOUT).usage(USAGE);

    for a in all_run_args() {
        s = s.arg(a);
    }

    commands.push(s);
}

fn all_run_args() -> Vec<Arg> {
    let mut run_args: Vec<Arg> = vec![];
    arg_run_commands(&mut run_args);

    run_args
}

fn arg_run_commands(args: &mut Vec<Arg>) {
    const SHORT: &str = "Commands passed to the target application";
    const LONG: &str = "
Commands specified as double-quoted strings are passed to the target
application within the bifrost container. Bifrost assumes that commands
passed explicitly take precedence over those specified in the manifest,
Bifrost.toml.

\t$ bifrost run --commands \"cd src/\" \"rg unwrap\"


";

    let a = Arg::with_name("commands")
        .takes_value(true)
        .multiple(true)
        .long("commands")
        .help(SHORT)
        .long_help(LONG);

    args.push(a);
}

fn sub_command_show(commands: &mut Vec<App>) {
    const ABOUT: &str = "Display files currently in the bifrost container";
    const USAGE: &str = "bifrost show [OPTIONS]";

    let mut s = SubCommand::with_name("show").about(ABOUT).usage(USAGE);

    for a in all_show_args() {
        s = s.arg(a);
    }

    commands.push(s);
}

fn all_show_args() -> Vec<Arg> {
    let mut show_args: Vec<Arg> = vec![];
    arg_show_all(&mut show_args);
    arg_show_diff(&mut show_args);
    show_args
}

fn arg_show_all(args: &mut Vec<Arg>) {
    const SHORT: &str = "Show all contents of the current workspace realm.";
    const LONG: &str = "
When `all` is passed, `bifrost show` will display the entire
contents of the current workspace realm. This command is
similar to running the `ls` command on unix-like platforms.


\t$ bifrost show --all


";

    let a = Arg::with_name("all")
        .long("all")
        .help(SHORT)
        .long_help(LONG);

    args.push(a);
}

fn arg_show_diff(args: &mut Vec<Arg>) {
    const SHORT: &str = "Show contents that have been modified locally.";
    const LONG: &str = "
When `diff` is passed, `bifrost show` will display the contents 
that have modified locally but have not been re-loaded into
the Bifrost container realm.


\t$ bifrost show --diff


";

    let a = Arg::with_name("diff")
        .long("diff")
        .help(SHORT)
        .long_help(LONG);

    args.push(a);
}

fn sub_command_load(commands: &mut Vec<App>) {
    const SHORT: &str = "Load directory, file, or files into the bifrost container";
    const USAGE: &str = "
    bifrost load [OPTIONS] --contents <contents>
    bifrost load [OPTIONS] --contents project/
    bifrost load [OPTIONS] --contents main.c avl.c bst.c
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
        .help(LONG)
        .long("contents")
        .long_help(LONG)
        .takes_value(true)
        .multiple(true);

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
