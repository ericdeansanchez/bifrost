#[cfg(test)]
extern crate bifrost;

use bifrost::util::template::EXPLICIT_LONG_HELP as LONG_HELP;

type App = clap::App<'static, 'static>;

struct TestApp {
    app: App,
}

impl TestApp {
    fn init() -> TestApp {
        TestApp {
            app: bifrost::core::app::cli(),
        }
    }
}

/// This is meant to be for demo purposes of what tests _can_ look like.
/// TODO: I don't think argument ordering (for display) is deterministic;
/// however, I believe `clap` allows manual ordering...
#[test]
fn test_basic_help() {
    let app = TestApp::init();
    // Passing bifrost --help because this panics when unwrap_err is called
    // so this is indicative of future error handling we must do to ensure
    // that unwrap is never called on a None value.
    assert!(test::compare_output(
        app.app,
        "bifrost --help",
        LONG_HELP,
        false
    ));
}

// This is here to demonstrate that we can do things to test the UI.
// Source:
// https://github.com/clap-rs/clap/blob/master/clap-test.rs
// MIT Copyright (c) 2015-2016 Kevin B. Knapp
#[cfg(test)]
mod test {
    use clap::App;
    use regex::Regex;
    use std::io::Cursor;
    use std::str;

    fn compare<S, S2>(l: S, r: S2) -> bool
    where
        S: AsRef<str>,
        S2: AsRef<str>,
    {
        let re = Regex::new("\x1b[^m]*m").unwrap();
        // Strip out any mismatching \r character on windows that might sneak in on either side
        let ls = l.as_ref().trim().replace("\r", "");
        let rs = r.as_ref().trim().replace("\r", "");
        let left = re.replace_all(&*ls, "");
        let right = re.replace_all(&*rs, "");
        let b = left == right;
        if !b {
            println!();
            println!("--> left");
            println!("{}", left);
            println!("--> right");
            println!("{}", right);
            println!("--")
        }
        b
    }

    pub fn compare_output(l: App, args: &str, right: &str, stderr: bool) -> bool {
        let mut buf = Cursor::new(Vec::with_capacity(50));
        let res = l.get_matches_from_safe(args.split(' ').collect::<Vec<_>>());
        let err = res.unwrap_err();
        err.write_to(&mut buf).unwrap();
        let content = buf.into_inner();
        let left = String::from_utf8(content).unwrap();
        assert_eq!(stderr, err.use_stderr());
        compare(left, right)
    }
}
