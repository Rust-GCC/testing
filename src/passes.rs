mod ast_export;
mod blake3;
mod gccrs_parsing;
mod gccrs_rustc_successes;
mod libcore;
mod rustc_dejagnu;

pub use ast_export::AstExport;
pub use blake3::Blake3;
pub use gccrs_parsing::GccrsParsing;
pub use gccrs_rustc_successes::GccrsRustcSuccesses;
pub use libcore::LibCore;
pub use rustc_dejagnu::RustcDejagnu;

use std::ffi::OsStr;
use std::fmt::Display;
use std::path::{Path, PathBuf};

use crate::compiler::Compiler;
use crate::{args::Args, error::Error};

/// Wrapper struct around an ftf test case. Ideally, this should be provided
/// directly by the ftf crate
#[derive(Debug)]
pub enum TestCase {
    Test {
        name: String,
        binary: String,
        exit_code: u8,
        timeout: i32,
        stderr: String,
        stdout: String,
        args: Vec<String>,
    },
    Skip,
}

impl Default for TestCase {
    fn default() -> Self {
        TestCase::Test {
            name: String::new(),
            binary: String::new(),
            exit_code: 0u8,
            // FIXME: Use duration here (#10)
            timeout: Self::DEFAULT_TIMEOUT,
            stderr: String::new(),
            stdout: String::new(),
            args: vec![],
        }
    }
}

impl TestCase {
    const DEFAULT_TIMEOUT: i32 = 15; // default timeout is 15 minutes

    pub fn from_compiler(mut compiler: Compiler) -> TestCase {
        let cmd = compiler.command();
        TestCase::default()
            .with_binary(cmd.get_program().to_string_lossy().to_string())
            .with_args(cmd.get_args().map(OsStr::to_string_lossy))
    }

    pub fn with_exit_code(mut self, new_exit_code: u8) -> TestCase {
        if let TestCase::Test {
            ref mut exit_code, ..
        } = self
        {
            *exit_code = new_exit_code;
        }

        self
    }

    pub fn with_timeout(mut self, new_timeout: i32) -> TestCase {
        if let TestCase::Test {
            ref mut timeout, ..
        } = self
        {
            *timeout = new_timeout;
        }

        self
    }

    pub fn with_name(mut self, new_name: String) -> TestCase {
        if let TestCase::Test { ref mut name, .. } = self {
            *name = new_name;
        }

        self
    }

    pub fn with_arg<T: Display>(mut self, arg: T) -> TestCase {
        if let TestCase::Test { ref mut args, .. } = self {
            args.push(arg.to_string());
        }

        self
    }

    pub fn with_args<T: Display>(self, args: impl Iterator<Item = T>) -> TestCase {
        args.fold(self, TestCase::with_arg)
    }

    pub fn with_binary<T: Display>(mut self, new_binary: T) -> TestCase {
        if let TestCase::Test { ref mut binary, .. } = self {
            *binary = new_binary.to_string();
        }

        self
    }
}

impl Display for TestCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestCase::Skip => Ok(()),
            TestCase::Test {
                name,
                binary,
                exit_code,
                timeout,
                stderr,
                stdout,
                args,
            } => {
                writeln!(f, "  - name: {name}")?;
                writeln!(f, "    binary: {binary}")?;
                writeln!(f, "    timeout: {timeout}")?;
                writeln!(f, "    exit_code: {exit_code}")?;
                writeln!(f, "    stdout: \"{stdout}\"")?;
                writeln!(f, "    stderr: \"{stderr}\"")?;
                writeln!(f, "    args:")?;

                for arg in args {
                    writeln!(f, "      - \"{arg}\"")?;
                }

                Ok(())
            }
        }
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
#[derive(Clone, Copy, clap::ValueEnum)]
pub enum PassKind {
    /// Generates test cases for running gccrs and rustc in parse-only mode on
    /// the rustc test suite
    GccrsParsing,
    /// Generates test cases for running rustc on gccrs' test-suite
    RustcDejagnu,
    /// Testsuite for running gccrs on valid rustc test cases
    GccrsRustcSucess,
    /// Testsuite for running gccrs on valid rustc test cases in #![no_std] mode
    GccrsRustcSucessNoStd,
    /// Testsuite for running gccrs on valid rustc test cases in #![no_core] mode
    GccrsRustcSucessNoCore,
    /// Compile the reference implementation of the Blake3 cryptographic algorithm
    Blake3,
    /// Compile the core library from various rust versions
    LibCore,
    /// Test our AST exporting algorithm on the whole gccrs testsuite
    AstExport,
}

#[derive(Debug)]
pub struct InvalidPassKind(String);

impl Display for InvalidPassKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid pass name provided: {}", self.0)
    }
}

impl Display for PassKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match &self {
            PassKind::GccrsParsing => "gccrs-parsing",
            PassKind::RustcDejagnu => "rustc-dejagnu",
            PassKind::GccrsRustcSucess => "gccrs-rustc-success",
            PassKind::GccrsRustcSucessNoStd => "gccrs-rustc-success-no-std",
            PassKind::GccrsRustcSucessNoCore => "gccrs-rustc-success-no-core",
            PassKind::Blake3 => "blake3",
            PassKind::LibCore => "libcore",
            PassKind::AstExport => "ast-export",
        };

        write!(f, "{s}")
    }
}
