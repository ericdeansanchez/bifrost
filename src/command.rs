//! # [IN_PROGRESS] Launch pad for primary functionality
use clap::ArgMatches;

use crate::workspace::WorkSpace;

pub fn load(workspace: &WorkSpace, contents: &ArgMatches<'static>) {
    // If contents specifies a valid directory, file, or files then load these
    // contents into the container. Otherwise, load the entire `WorkSpace`
    let args = contents.values_of("contents");
    println!("[STUB] command::load() contents: {:?}", args)
}

pub fn unload() {
    println!("[STUB] command::unload()");
}

pub fn show(workspace: &WorkSpace) {
    println!("[INPROGRESS] command::show()");
    workspace.display();
}

pub fn run() {
    println!("[STUB] command::run()");
}
