use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{args::Args, error::Error};

/// Wrapper struct around an ftf test case. Ideally, this should be provided
/// directly by the ftf crate
#[derive(Default, Debug)]
pub struct TestCase {
    name: String,
    binary: String,
    exit_code: i32,
    timeout: i32,
    args: Vec<String>,
}

impl TestCase {
    pub fn with_exit_code(self, exit_code: i32) -> TestCase {
        TestCase { exit_code, ..self }
    }

    pub fn with_timeout(self, timeout: i32) -> TestCase {
        TestCase { timeout, ..self }
    }

    pub fn with_name(self, name: String) -> TestCase {
        TestCase { name, ..self }
    }

    pub fn with_arg<T: Display>(self, arg: T) -> TestCase {
        let mut new_args = self.args;
        new_args.push(arg.to_string());

        TestCase {
            args: new_args,
            ..self
        }
    }

    pub fn with_binary<T: Display>(self, binary: T) -> TestCase {
        TestCase {
            binary: binary.to_string(),
            ..self
        }
    }
}

impl Display for TestCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  - name: {}", self.name)?;
        writeln!(f, "    binary: {}", self.binary)?;
        writeln!(f, "    timeout: {}", self.timeout)?;
        writeln!(f, "    exit_code: {}", self.exit_code)?;
        writeln!(f, "    args:")?;

        for arg in self.args.iter() {
            write!(f, "      - \"{}\"", arg)?;
        }

        Ok(())
    }
}

pub trait Pass: Sync {
    /// Fetch test cases
    fn fetch(&self, args: &Args) -> Result<Vec<PathBuf>, Error>;

    /// Adapt test cases, running any kind of transformation on them and providing
    /// extra information necessary for the test case generation
    fn adapt(&self, args: &Args, file: &Path) -> Result<TestCase, Error>;
}

/// Passes to run when generating the test-suite file. One can chose to run only
/// a specific pass, or multiple of them
pub enum PassKind {
    /// Generates test cases for running gccrs and rustc in parse-only mode on
    /// the rustc test suite
    GccrsParsing,
    /// Generates test cases for running rustc on gccrs' test-suite
    RustcDejagnu,
}

#[derive(Debug)]
pub struct InvalidPassKind(String);

impl Display for InvalidPassKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid pass name provided: {}", self.0)
    }
}

impl FromStr for PassKind {
    type Err = InvalidPassKind;

    fn from_str(s: &str) -> Result<PassKind, Self::Err> {
        match s {
            "gccrs-parsing" => Ok(PassKind::GccrsParsing),
            "rustc-dejagnu" => Ok(PassKind::RustcDejagnu),
            s => Err(InvalidPassKind(s.to_string())),
        }
    }
}
