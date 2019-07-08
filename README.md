# Bifröst
_The_ Bifröst is a burning rainbow bridge that reaches between Midgard (Earth) 
and Asgard, the realm of the gods.

_This_ Bifröst seeks to be a bridge to other command line tools; really it is
currently being designed to bridge the gap between one tool in particular. 

[![Build Status](https://travis-ci.org/ericdeansanchez/bifrost.svg?branch=master)](https://travis-ci.org/ericdeansanchez/bifrost)

## Overview

Bifröst seeks to be a convenience layer between you and a containerized command
line application. The general idea is to be a bridge between one OS, say
macOS Mojave, and another OS––Ubuntu.

Bifröst is currently a **work in progress** and is progressing towards an actual
release (as opposed to the pre-release being offered now).

## Contents

* [Requirements](#requirements)
* [Installation](#installation)
  * [Setup](#setup)
* [Example](#example)
* [Contributing](#contributing)

## Requirements

Bifröst depends on **[docker](https://www.docker.com/get-started)** and
requires that docker is installed. Perhaps, in the future, bifröst can be more
flexible with the container engines it supports.

While running **macOS** is not necessarily a requirement, development and
testing has been primarily done on this OS. Ensuring Windows compatibility is
next on the todo list.

## Installation

Bifröst is still under its initial development, however, a pre-release version is
available via:

```
  $ brew install ericdeansanchez/heimdallr/bifrost
```
**Warning**: see the note below.

#### Note
This release is only meant to keep development rolling forward and
* it is not _necessarily_ intended for general use and
* it is not _necessarily_ ready for general use

## Setup

Once bifröst has been installed, there is one last step to perform before it can
be used:

Ensure you have the latest bifröst version:

```bash
$ bifrost --version
```
This next part can take a little while, but that's because it has some `build`ing
to do. Run the following command and take a little break or something (get
up and stretch or check hacker news).

Run the `setup` command:
```bash
$ bifrost setup
```

This will create the following directory tree:
```text
.bifrost
└── container
    └── bifrost
        └── Dockerfile
```

It isn't necessary to know this. However, it can be hard to know exactly what
apps are doing, where they do it, etc. So this is included for the purposes of
transparency.

## Example

Now that everything has been set up, you can start using the bifröst.

```bash
$ mkdir example
```

```bash
$ cd example
```

```bash
$ touch main.c
```

Using your editor of choice:

In `example/main.c`:
```c
#include <stdio.h>

int main() {
  printf("hello world\n");
  return 0;
}
```
Now, for whatever reason you don't want to compile and run this program on your
system, but instead you want to run it on ubuntu:16.04:

Initial the current working directory as a bifröst realm:

In `example/`:
```bash
$ bifrost init
```
If everything went well you should see the following message displayed:

```text
initialized default bifrost realm in ../../example
```
Great! The directory tree should look like this:

```text
example/
├── Bifrost.toml
└── main.c
```

Now open the `Bifrost.toml` manifest for editing. The default manifest looks
something like this:

```toml
[project]
name = "project name"

[container]
name = "docker"

[workspace]
name = "example"
ignore = ["target", ".git", ".gitignore"]

[command]
cmds = ["command string(s)"]
```
Now, `"command string(s)"` is not a valid command so change this to:

```toml
[command]
cmds = [
  "cd example",
  "gcc main.c -o main",
  "./main"
]
```
Awesome, now lets `load` up our container:

```bash
$ bifrost load
```
The corresponding output should be something like:

```text
bifrost: loaded {278} bytes from realm {example}
```

And now it's time to `run` our program. If the docker desktop is not currently
up and running, bifröst will start it up for you and proceed to run your program
with the supplied `cmds`.

Let's try it:

```bash
$ bifrost run
```

The corresponding output should be something like:

```text
stdout:
hello world

stderr:
```
Success!

## What Happened?

Bifröst is executing commands in the context of the running container. Our `cmds`
relied on the container knowing what `cd` and `gcc` were (that they were installed
and executable). 

We can change our commands, `cmds`, to the following:

```toml
[command]
cmds = [
  "gcc --version"
]
```

And the corresponding output would be:
```text
stdout:
gcc (Ubuntu 5.4.0-6ubuntu1~16.04.11) 5.4.0 20160609
```

# Contributing

Contributions are welcome! No contribution is too small––bug fix, a new feature,
or a typo fix––all are welcome.

* contribution [template]()
* issue [template]()

## Prerequisites

Bifröst is written in Rust so make sure you have [Rust installed](https://www.rust-lang.org/tools/install.)


## Clone

Clone the repository:

```bash
$ git clone https://github.com/ericdeansanchez/bifrost.git
```

## Build

cd into the repository and run:

```bash
$ cargo build
```

## Test

Ensure the tests pass on your system (please open an issue if they do not):

```bash
$ cargo test
```

## Read 

These are the docs you really want. Copy & paste the command below in your 
terminal. Make sure `--no-deps` is passed otherwise you'll build documentation 
for all/any dependencies.

```bash
$ cargo doc --no-deps --open
```

## Run

```bash
$ cargo run init
$ cargo run load
$ cargo run show
$ cargo run run
$ cargo run unload
```
