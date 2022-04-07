use std::path::{Path, PathBuf};

use crate::args::Args;
use crate::copy_rs_files;
use crate::error::Error;
use crate::passes::{Pass, TestCase};

pub struct GccrsPrivacy;

impl Pass for GccrsPrivacy {
    fn fetch(&self, args: &Args) -> Result<Vec<PathBuf>, Error> {
        let rust_path = &args.rust_path;
        let test_cases = rust_path
            .join("src")
            .join("test")
            .join("ui")
            .join("privacy");

        let mut privacy_files = copy_rs_files(&test_cases, &args.output_dir, rust_path)?;

        let test_cases = rust_path.join("src").join("test").join("ui").join("pub");

        privacy_files.append(&mut copy_rs_files(
            &test_cases,
            &args.output_dir,
            rust_path,
        )?);

        Ok(privacy_files)
    }

    fn adapt(&self, args: &Args, file: &Path) -> Result<TestCase, Error> {
        todo!("parse given file for `//~ ERROR` and adapt to dejagnu directives, emitting test cases with dejagnu as a binary")
    }
}
