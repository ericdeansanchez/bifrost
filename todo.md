# Todo

I really want to keep working (and learning) on bifröst,
but I need to pay the bills as well. So if bifröst lacks
something you want check here to see if it's planned.

Otherwise this will be the place where I talk about what
I plan to implement/fix.

## Thu Jul 11 11:24:30 PDT 2019

#### Dockerfile Configuration

I want this to be as convenient and sensible as possible.
I'd like to be able to edit this file and rebuild as I
want/need. So this translates to (probably) two branches:

* ```rust 
    fn rebuild
    ```

* ```rust
    fn add...
    fn remove...
    ```

Since no one, except myself, uses bifröst I imagine I have
time.

The other things I need/want to do is clean up some things
I know need the TLC. Namely,

* I feel that `config.rs` is a bit of a mess at the moment.
* `main.rs` has been growing along in an ad-hoc way (it works
  but it's ugly as hell...)