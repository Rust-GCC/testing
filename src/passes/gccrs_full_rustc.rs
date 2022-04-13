use crate::args::Args;
use crate::copy_rs_files;
use crate::error::Error;
use crate::passes::{Pass, TestCase};

use std::fs;
use std::path::{Path, PathBuf};

pub struct GccrsFullRustc;

impl Pass for GccrsFullRustc {
    fn fetch(&self, args: &Args) -> Result<Vec<PathBuf>, Error> {
        // FIXME: Do we want more tests than just src/test/ui?
        let rust_path = &args.rust_path;
        let ui_tests = rust_path.join("src").join("test").join("ui");

        // FIXME: Do we have to filter some test files here that cause a gccrs
        // crash or an ftf crash?

        copy_rs_files(&ui_tests, &args.output_dir, rust_path)
    }

    fn adapt(&self, args: &Args, file: &Path) -> Result<TestCase, Error> {
        let test_content = fs::read_to_string(file)?;

        let exit_code = if test_content.contains("ERROR") { 1 } else { 0 };

        let test_case = TestCase::default()
            .with_name(format!("Compile `{}`", file.display()))
            .with_binary(args.gccrs.display())
            .with_exit_code(exit_code)
            .with_timeout(5)
            .with_arg(file.display());

        // FIXME: We need to add a lot more cases here and basically write a translator
        // from rustc error to gccrs errors
        let test_case = if exit_code == 1 {
            test_case.with_stderr("error")
        } else {
            test_case
        };

        Ok(test_case)
    }
}

// SAFETY: No data is kept in the struct, so we can safely use it across threads
unsafe impl Sync for GccrsFullRustc {}
