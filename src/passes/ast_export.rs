use std::fs;
use std::path::{Path, PathBuf};

use crate::args::Args;
use crate::compiler::{Compiler, Kind};
use crate::error::Error;
use crate::fetch_rust_files;
use crate::passes::{Pass, TestCase};

fn get_original_file_from_pretty(pretty_file: &Path) -> PathBuf {
    let mut original_file = pretty_file.to_owned().with_extension("rs");
    original_file.set_extension("rs");

    assert!(original_file.exists());

    original_file
}

fn adapt_compilation(args: &Args, pretty_file: &Path) -> Result<TestCase, Error> {
    let original_file = get_original_file_from_pretty(pretty_file);

    let is_valid = Compiler::new(Kind::Crab1, args)
        .command()
        .arg(original_file.as_os_str())
        .status()?
        .success();

    // what we want to do is:
    // if the file compiles, then we want its prettified version to compile as well
    // if the file does not compile, then we want the same errors as the original on the prettified file
    // but this is waaaaay harder to do :( and probably not worth it

    let test_case = if is_valid {
        TestCase::from_compiler(Compiler::new(Kind::Crab1, args))
            .with_name(format!("Compile prettified `{}`", original_file.display()))
            .with_exit_code(0)
            .with_arg(pretty_file.display())
    } else {
        TestCase::Skip
    };

    Ok(test_case)
}

pub enum AstExport {
    Compile,
}

impl Pass for AstExport {
    fn fetch(&self, args: &Args) -> Result<Vec<PathBuf>, Error> {
        let gccrs_path = &args.gccrs_path;
        let tests_path = gccrs_path.join("gcc").join("testsuite").join("rust");
        let output_dir = args.output_dir.join("ast-export");

        // For each file:
        //      gccrs -frust-dump-ast-pretty <file>
        //      cp gccrs.ast-pretty.dump <new_path>
        fetch_rust_files(&tests_path)
            // FIXME: Cannot parallelize this since the AST dump is always the same file...
            // Think about -frust-dump-ast-pretty=<file>?
            .into_iter()
            .map(|entry| {
                let new_path_original = output_dir.join(entry.path());
                let new_path = output_dir.join(entry.path()).with_extension("pretty-rs");

                Compiler::new(Kind::Crab1, args)
                    .command()
                    .arg(entry.path())
                    .arg("-frust-dump-ast-pretty")
                    .status()?;

                // Make sure the directory exists
                if let Some(parent) = new_path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)?;
                    }
                }

                fs::copy(entry.path(), new_path_original)?;
                fs::copy("gccrs.ast-pretty.dump", &new_path)?;

                Ok(new_path)
            })
            .collect()
    }

    fn adapt(&self, args: &Args, pretty_file: &Path) -> Result<TestCase, Error> {
        match self {
            AstExport::Compile => adapt_compilation(args, pretty_file),
        }
    }
}
