use std::{fmt::Display, path::PathBuf, str::FromStr};

use crate::{args::Args, error::Error};

pub struct TestCase {
    name: String,
    binary: String,
    exit_code: i32,
    timeout: i32,
    args: Vec<String>,
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

pub trait Pass {
    /// Extra information to extract from the adapting function and give to the
    /// generate function
    type ExtraInfo;

    /// Fetch test cases
    fn fetch(args: &Args) -> Result<Vec<PathBuf>, Error>;

    /// Adapt test cases, running any kind of transformation on them and providing
    /// extra information necessary for the test case generation
    fn adapt(files: &[PathBuf]) -> Result<Vec<(PathBuf, Self::ExtraInfo)>, Error>;

    /// Generate the test case from the file and extra information
    fn generate(files: &[(PathBuf, Self::ExtraInfo)]) -> Result<Vec<TestCase>, Error>;
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
