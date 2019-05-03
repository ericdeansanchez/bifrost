# Bifröst
_The_ Bifröst is a burning rainbow bridge that reaches between Midgard (Earth) 
and Asgard, the realm of the gods.

_This_ Bifröst seeks to be a bridge to other command line tools; really it is
currently being designed to bridge the gap between one tool in particular. 

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

There is some default behavior for running Bifrost this way:

```bash
$ cargo run
```

## Running the actual binary

Bifrost is currently a work in progress and running it looks something like:

```bash
$ ./target/debug/bifrost show
$ ./target/debug/bifrost run
```
