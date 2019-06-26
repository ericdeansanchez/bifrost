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

For example, I have this command line app that I cannot use on macOS Mojave, but
it is a breeze to run on linux. I don't want to dual-boot, I don't want to run anything
in a virtual machine, and I don't want to rely on university lab machines.

With Bifröst, I want to be able to containerize a command line app, automate file
transfering (loading, reloading, unloading) and run a sequence of commands without
so much command-line hand-jamming.

Bifröst is currently a **work in progress** and is not currently ready for use.

## Installation

Bifröst is still under initial development, however, a pre-release version is
available via:

```
  $ brew install bifrost
```
**Warning**: see the note below.

#### Note
This release is only meant to keep development rolling forward and
* it is **NOT** intended for general use and
* it is **NOT** ready for general use

Currently, Wed Jun 26 16:52:28 PDT 2019, it lacks some external/supporting directories and files
necessary for persistence and basic operation.



## Build

```bash
$ cargo build
```

## Test
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

Bifrost is currently a work in progress and running it looks something like:

```bash
$ cargo run init
$ cargo run load
$ cargo run show
$ cargo run run
$ cargo run unload
```
