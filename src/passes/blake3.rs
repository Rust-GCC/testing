use std::fs;
use std::path::{Path, PathBuf};

use crate::args::Args;
use crate::compiler::{Compiler, CrateType, Kind};
use crate::error::Error;
use crate::passes::{Pass, TestCase};

/// Taken directly from [the Blake3 Rust reference implementation](https://github.com/BLAKE3-team/BLAKE3/blob/master/reference_impl/reference_impl.rs)
/// commit: da4c792
const BLAKE3_TEMPLATE: &str = include_str!("blake3_template");

// TODO: We can think about having an extra template for a "modified" version
// of Blake3 with our little core prelude and lang items added to it

pub enum Blake3 {
    GccrsOriginal,
    GccrsPrelude,
    RustcNoStd,
    RustcNoCore,
}

impl Blake3 {
    /// All variants of the [`Blake3`] enum
    pub fn variants() -> Vec<Blake3> {
        vec![
            Blake3::GccrsOriginal,
            Blake3::GccrsPrelude,
            Blake3::RustcNoStd,
            Blake3::RustcNoCore,
        ]
    }

    /// File name suffix to append when creating the file for each test case
    fn suffix(&self) -> &'static str {
        match self {
            Blake3::GccrsOriginal => "original",
            Blake3::GccrsPrelude => "gccrs-prelude",
            Blake3::RustcNoStd => "rustc-no-std",
            Blake3::RustcNoCore => "rustc-no-core",
        }
    }
}

impl Pass for Blake3 {
    fn fetch(&self, args: &Args) -> Result<Vec<PathBuf>, Error> {
        let output_file = args
            .output_dir
            .clone()
            .join(format!("blake3-{}.rs", self.suffix()));

        // We can create empty files for now as we will adapt them afterwards
        // in the next phase. What matters is that they have a different name
        // based on the variant we're currently creating a test case for

        Ok(vec![output_file])
    }

    fn adapt(&self, args: &Args, file: &Path) -> Result<TestCase, Error> {
        let prelude = match self {
            Blake3::GccrsOriginal => "",
            Blake3::RustcNoStd => "#![no_std]\n",
            Blake3::GccrsPrelude => "lang_item prelude!",
            Blake3::RustcNoCore => "#![feature(no_core)]\n#![no_core]\n + lang_item prelude!",
            // FIXME: Missing lang items prelude
        };

        let compiler = match self {
            Blake3::GccrsOriginal | Blake3::GccrsPrelude => Compiler::new(Kind::Rust1, args),
            Blake3::RustcNoStd | Blake3::RustcNoCore => Compiler::new(Kind::RustcBootstrap, args),
        }
        .crate_type(CrateType::Library);

        fs::write(file, format!("{prelude}{BLAKE3_TEMPLATE}"))?;

        Ok(TestCase::from_compiler(compiler)
            .with_arg(file.display())
            .with_name(format!(
                "Compile Blake3 reference implementation ({})",
                self.suffix()
            ))
            .with_exit_code(0))
    }
}
