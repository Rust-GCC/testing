use crate::args::Args;
use crate::compiler::{Compiler, Kind};
use crate::copy_rs_files;
use crate::error::Error;
use crate::passes::{Pass, TestCase};

use std::fs;
use std::path::{Path, PathBuf};

pub struct RustcDejagnu;

impl Pass for RustcDejagnu {
    fn fetch(&self, args: &Args) -> Result<Vec<PathBuf>, Error> {
        let gccrs_path = &args.gccrs_path;
        let tests_path = gccrs_path.join("gcc").join("testsuite").join("rust");

        copy_rs_files(&tests_path, &args.output_dir, gccrs_path)
    }

    fn adapt(&self, args: &Args, file: &Path) -> Result<TestCase, Error> {
        let test_content = fs::read_to_string(file)?;

        let exit_code = u8::from(test_content.contains("dg-error"));

        // FIXME: This should be removed once we have a proper main shim in gccrs
        // This is to make sure that we can't ever get a "success" because a test
        // contains a dg-error directive and a `fn main() -> i32` so rustc produces
        // the correct exit code
        let exit_code = if test_content.contains("fn main() -> i32") {
            255
        } else {
            exit_code
        };

        let test_case = TestCase::from_compiler(Compiler::new(Kind::RustcBootstrap, args))
            .with_name(format!("Run rustc on `{}`", file.display()))
            .with_exit_code(exit_code)
            .with_timeout(5)
            .with_arg(file.display())
            .with_arg("-o") // Compile all files to the same executable name to avoid having to clean up 500 executables...
            .with_arg("rustc_out");

        Ok(test_case)
    }
}
