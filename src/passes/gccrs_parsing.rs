use crate::args::Args;
use crate::copy_rs_files;
use crate::error::Error;
use crate::passes::{Pass, TestCase};

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct GccrsParsing;

impl Pass for GccrsParsing {
    fn fetch(&self, args: &Args) -> Result<Vec<PathBuf>, Error> {
        let rust_path = &args.rust_path;
        let ui_tests = rust_path.join("src").join("test");

        copy_rs_files(&ui_tests, &args.output_dir, rust_path)
    }

    fn adapt(&self, args: &Args, file: &Path) -> Result<TestCase, Error> {
        let is_valid = Command::new(&args.rustc)
            // FIXME: We need to instead build a specific version of rustc to test against rather than using the user's
            // FIXME: We can maybe instead use the rustc-ap-rustc_parse crate which would be much faster
            .env("RUSTC_BOOTSTRAP", "1")
            .arg("-Z")
            .arg("parse-only")
            .arg("--edition")
            .arg("2021")
            .arg(file.as_os_str())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .status()?
            .success();

        let test_case = TestCase::default()
            .with_name(format!("Parse `{}`", file.display()))
            .with_binary(args.gccrs.display())
            .with_exit_code(if is_valid { 0 } else { 1 })
            .with_timeout(5)
            .with_arg("-fsyntax-only")
            .with_arg(file.display());

        Ok(test_case)
    }
}

// SAFETY: No data is kept in the struct, so we can safely use it across threads
unsafe impl Sync for GccrsParsing {}
