use crate::args::Args;
use crate::compiler::{Compiler, Edition, Kind};
use crate::copy_rs_files;
use crate::error::Error;
use crate::passes::{Pass, TestCase};

use std::path::{Path, PathBuf};

pub struct GccrsParsing;

impl Pass for GccrsParsing {
    fn fetch(&self, args: &Args) -> Result<Vec<PathBuf>, Error> {
        let rust_path = &args.rust_path;
        let ui_tests = rust_path.join("src").join("test");

        copy_rs_files(&ui_tests, &args.output_dir, rust_path)
    }

    fn adapt(&self, args: &Args, file: &Path) -> Result<TestCase, Error> {
        let is_valid = Compiler::new(Kind::RustcBootstrap, args)
            .edition(Edition::E2021)
            .command()
            // FIXME: We need to instead build a specific version of rustc to test against rather than using the user's
            // FIXME: We can maybe instead use the rustc-ap-rustc_parse crate which would be much faster
            .arg("-Z")
            .arg("parse-only")
            .arg(file.as_os_str())
            .status()?
            .success();

        let test_case = TestCase::from_compiler(Compiler::new(Kind::Rust1, args))
            .with_name(format!("Parse `{}`", file.display()))
            .with_exit_code(u8::from(!is_valid))
            .with_timeout(1)
            .with_arg("-fsyntax-only")
            .with_arg(file.display());

        Ok(test_case)
    }
}
