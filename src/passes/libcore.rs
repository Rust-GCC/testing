use std::path::{Path, PathBuf};

use crate::args::Args;
use crate::copy_rs_files;
use crate::error::Error;
use crate::passes::{Pass, TestCase};

pub enum LibCore {
    // libcore from rustc 1.49.0
    V149,
}

impl LibCore {
    fn as_str(&self) -> &str {
        match self {
            LibCore::V149 => "1.49",
        }
    }
}

impl Pass for LibCore {
    fn fetch(&self, args: &Args) -> Result<Vec<PathBuf>, Error> {
        let rust_path = &args.rust_path;
        let core_path = rust_path.join("library").join("core");

        // TODO: How do we ensure the rustc submodule contains 1.49.0 libcore?
        // Should we keep it as a template? Should we do some `git checkout` horrors
        // here?

        copy_rs_files(&core_path, &args.output_dir, rust_path)?;

        // We only want to compile a single file, and the others as modules
        Ok(vec![args
            .output_dir
            .join(core_path)
            .join("src")
            .join("lib.rs")])
    }

    fn adapt(&self, args: &Args, file: &Path) -> Result<TestCase, Error> {
        Ok(TestCase::new()
            .with_name(format!("Compiling libcore {}", self.as_str()))
            .with_binary(args.gccrs.display())
            .with_arg(file.display())
            .with_exit_code(0))
    }
}
