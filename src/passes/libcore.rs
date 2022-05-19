use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::args::Args;
use crate::copy_rs_files;
use crate::error::{Error, MiscKind};
use crate::passes::{Pass, TestCase};

pub enum LibCore {
    // libcore from rustc 1.49.0
    V149,
}

impl LibCore {
    fn tag(&self) -> &str {
        match self {
            LibCore::V149 => "1.49.0",
        }
    }
}

impl Pass for LibCore {
    fn fetch(&self, args: &Args) -> Result<Vec<PathBuf>, Error> {
        let rust_path = &args.rust_path;
        let core_path = rust_path.join("library").join("core");

        let map_checkout = |success, arg_string| {
            if success {
                Ok(())
            } else {
                Err(Error::Misc(MiscKind::Git { arg_string }))
            }
        };

        let rust_git = |args: Vec<&str>| {
            let old_dir = env::current_dir()?;
            env::set_current_dir(rust_path)?;

            let res = Command::new("git").args(&args).status()?;

            env::set_current_dir(old_dir)?;

            map_checkout(res.success(), args.join(" "))
        };

        rust_git(vec!["checkout", self.tag()])?;

        copy_rs_files(&core_path, &args.output_dir, rust_path)?;

        rust_git(vec!["switch", "-"])?;

        // We only want to compile a single file, and the others as modules
        Ok(vec![args
            .output_dir
            .join(core_path)
            .join("src")
            .join("lib.rs")])
    }

    fn adapt(&self, args: &Args, file: &Path) -> Result<TestCase, Error> {
        Ok(TestCase::new()
            .with_name(format!("Compiling libcore {}", self.tag()))
            .with_binary(args.gccrs.display())
            .with_arg(file.display())
            .with_exit_code(0))
    }
}
