use std::fs;
use std::path::{Path, PathBuf};

use crate::args::Args;
use crate::error::Error;
use crate::passes::{Pass, TestCase};

/// Taken directly from [the Blake3 Rust reference implementation](https://github.com/BLAKE3-team/BLAKE3/blob/master/reference_impl/reference_impl.rs)
/// commit: da4c792
const BLAKE3_TEMPLATE: &str = include_str!("blake3_template");

// TODO: We can think about having an extra template for a "modified" version
// of Blake3 with our little core prelude and lang items added to it

pub struct Blake3;

impl Pass for Blake3 {
    fn fetch(&self, args: &Args) -> Result<Vec<PathBuf>, Error> {
        let output_file = args.output_dir.clone().join("blake3_original.rs");

        fs::write(&output_file, BLAKE3_TEMPLATE)?;

        Ok(vec![output_file])
    }

    fn adapt(&self, args: &Args, file: &Path) -> Result<TestCase, Error> {
        Ok(TestCase::new()
            .with_arg(file.display())
            .with_name(String::from("Compile Blake3 reference implementation"))
            .with_exit_code(0)
            .with_binary(args.gccrs.display()))
    }
}
