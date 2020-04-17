# Bifröst Architecture

At a high level, Bifröst aims to make working with containerized command line apps
easier. It is a work-in-progress and it works **for me**. Here is an overview of the general architecture that supports this goal.

## High Level Project Structure
```text
src/
├── bifrost
│   ├── core # The core data structures and supporting methods.
│   ├── lib.rs
│   ├── ops  # Implementation details for the bifrost operations (init, load, show, run, unload).
│   └── util # Supporting utilities.
└── bin
    └── bifrost
        ├── commands # Contains the modules responsible for `exec`uting bifrost operations.
        │   ├── init.rs
        │   ├── load.rs
        │   ├── mod.rs
        │   ├── run.rs
        │   ├── show.rs
        │   └── unload.rs
        └── main.rs  # Entry point
```

## Entry point @ src/bin/bifrost/main.rs
The main entry point is [main.rs](src/bin/bifrost/main.rs). Where `match`ed subcommands
are `exec`uted. 

## (Sub) Commands

Bifröst is a binary command line app that utilizes [clap](https://clap.rs)––"a command 
line argument parser for Rust." The functionality clap provides more than suits
Bifröst's needs. 

Subcommands are built up in [app.rs](src/bifrost/core/app.rs) in a iterative
fashion. Clap has been influential in allowing Bifröst reach its goals of being
extensible, modular, and straightforward.

All subcommands are registered with the top-level application via the `all_sub_commands`
function. In turn, each subcommand registers its supported arguments via `all_load_args`,
`all_show_args`, etc. functions. The process is slightly repetitive, but this means
there are plenty of examples to draw from when new functionality needs to be added.

Each subcommand is executed with a `Config` and a set of `clap::ArgMatches`. From these
two structures each command constructs a workspace that is `BifrostOperable`.

## Data Structures

#### [Config](src/bifrost/core/config.rs) @ src/bifrost/core/config.rs

A `Config` contains part (or all) of the information necessary to construct a
`WorkSpace`, namely a user's home path, current working directory path, and a
manifest.

Each Bifröst realm is initialized with a `Bifrost.toml` manifest. This manifest
provides a means of persistence run-to-run and also gives a user the option to
specify command line arguments to their target application.

A manifest is constructed either from **only** a `Bifrost.toml` file or from the
combination of command line arguments and the existing `Bifrost.toml` manifest.

The `Config` structure provides the necessary information needed to construct
`WorkingDir`s that ultimately make up a `WorkSpace`'s contents.

#### [WorkingDir](src/bifrost/core/workingdir.rs) @ src/bifrost/core/workingdir.rs

`WorkingDir`s provide a way to structure a representaion of a user's source directory
with the ability to ignore unwanted files so that they are excluded from the
Bifröst container.

This structure leverages [@burnsushi](https://github.com/BurntSushi)'s
[walkdir](https://github.com/BurntSushi/walkdir) crate to walk directories.

These `WorkingDir`s make up the contents of a `WorkSpace`.

#### [WorkSpace](src/bifrost/core/workspace.rs) @ src/bifrost/core/workspace.rs

`WorkSpace`s take ownership of a `Config` and use it to construct its 
`contents: Option<Vec<WorkingDir>>`. `WorkSpace`s take orders from spaces that
are actually `BifrostOperable`.

#### Bifrost-Operable Spaces

Spaces that are `BifrostOperable` implement `prep`, `build`, and `exec`. Bifrost 
operations can only operate on workspaces that have made it far enough to be owned
by `BifrostOperable` spaces.

The following space are `BifrostOperable`:

* `LoadSpace` - structure that supports `bifrost load`
* `UnloadSpace` - structure that supports `bifrost unload`
* `ShowSpace` - structure that support `bifrost show`
* `RunSpace` - structure that supports `bifrost run`

Each space above is `prep`able, `build`able, and `exec`utable. As a result, they 
also provide a way to control _how_ a `WorkSpace` is constructed.

The division of spaces into the above categories is to prevent the wrong operation
being executed on a workspace that may or may not support it. This way, we know
that spaces support the operations they were constructed for.
