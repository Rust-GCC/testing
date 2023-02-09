use crate::args::Args;
use crate::compiler::{Compiler, Edition, Kind};
use crate::copy_rs_files;
use crate::error::Error;
use crate::passes::{Pass, TestCase};

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use wait_timeout::ChildExt;

pub enum GccrsRustcSuccesses {
    Full,
    NoStd,
    NoCore,
}

impl Display for GccrsRustcSuccesses {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let s = match self {
            GccrsRustcSuccesses::Full => "",
            GccrsRustcSuccesses::NoStd => "no-std",
            GccrsRustcSuccesses::NoCore => "no-core",
        };

        write!(f, "{s}")
    }
}

impl Pass for GccrsRustcSuccesses {
    fn fetch(&self, args: &Args) -> Result<Vec<PathBuf>, Error> {
        // FIXME: Do we want more tests than just src/test/ui?
        let rust_path = &args.rust_path;
        let ui_tests = rust_path.join("src").join("test").join("ui");

        let out_dir = match self {
            GccrsRustcSuccesses::Full => args.output_dir.clone(),
            // We need different output files since we're going to modify files for these
            GccrsRustcSuccesses::NoStd => args.output_dir.join("no-std"),
            GccrsRustcSuccesses::NoCore => args.output_dir.join("no-core"),
        };

        copy_rs_files(&ui_tests, &out_dir, rust_path)
    }

    fn adapt(&self, args: &Args, file: &Path) -> Result<TestCase, Error> {
        let test_content = fs::read_to_string(file)?;

        // FIXME: We should also see if the file contains a main function maybe?
        // To know if we can pass `--crate-type=lib`?

        // We're only interested in successes
        if test_content.contains("ERROR") {
            return Ok(TestCase::Skip);
        }

        let extra_str = match self {
            GccrsRustcSuccesses::Full => "",
            GccrsRustcSuccesses::NoStd => "#![no_std]\n",
            GccrsRustcSuccesses::NoCore => "#![feature(no_core)]\n#![no_core]\n",
        };

        fs::write(file, format!("{extra_str}{test_content}"))?;

        if let GccrsRustcSuccesses::NoStd | GccrsRustcSuccesses::NoCore = self {
            let mut child = Compiler::new(Kind::RustcBootstrap, args)
                .edition(Edition::E2021)
                .crate_name("rustc_output")
                .command()
                .arg("--crate-type")
                .arg("lib")
                .arg(file.as_os_str())
                .spawn()?;

            let is_valid = if let Some(status) = child.wait_timeout(Duration::from_secs(30))? {
                status.success()
            } else {
                child.kill()?;
                false
            };

            if !is_valid {
                return Ok(TestCase::Skip);
            }
        }

        let test_case = TestCase::from_compiler(Compiler::new(Kind::Rust1, args))
            .with_name(format!("Compile {} success `{}`", self, file.display()))
            .with_exit_code(0)
            // FIXME: Use proper duration here (#10)
            .with_timeout(5 * 60) // ftf's timeout is in seconds, so 5 minutes
            .with_arg(file.display());

        Ok(test_case)
    }
}
