use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use wait_timeout::ChildExt;

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

    let is_valid = Compiler::new(Kind::Rust1, args)
        .command()
        .arg(original_file.as_os_str())
        .status()?
        .success();

    let test_case = TestCase::from_compiler(Compiler::new(Kind::Rust1, args))
        .with_name(format!("Compile prettified `{}`", original_file.display()))
        .with_exit_code(u8::from(!is_valid))
        .with_arg(pretty_file.display());

    Ok(test_case)
}

fn adapt_run(args: &Args, pretty_file: &Path) -> Result<TestCase, Error> {
    let original_file = get_original_file_from_pretty(pretty_file);
    let binary_name = original_file.with_extension("");

    // Build the original binary
    if !Compiler::new(Kind::Rust1, args)
        .command()
        .arg(original_file.as_os_str())
        .arg("-o")
        .arg(binary_name.as_os_str())
        .status()?
        .success()
    {
        // This will be handled by the `AstExport::Compile` part
        return Ok(TestCase::Skip);
    }

    let mut child = Command::new(binary_name.as_os_str())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()?;

    // Run the original binary
    let binary_exit_code = if let Some(exit_status) = child.wait_timeout(Duration::from_secs(5))? {
        exit_status.code()
    } else {
        child.kill()?;
        return Ok(TestCase::Skip);
    };

    match binary_exit_code {
        None => Ok(TestCase::Skip),
        Some(code) => {
            let binary_name = binary_name.with_extension("pretty");
            // We now build the "prettified binary". If that fails, skip it as that's been handled by the `Compile` phase
            if !Compiler::new(Kind::Rust1, args)
                .command()
                .arg(pretty_file)
                .arg("-o")
                .arg(binary_name.as_os_str())
                .status()?
                .success()
            {
                return Ok(TestCase::Skip);
            }

            // TODO: Should we also check that the output is the same?
            // TODO: Maybe just for stdout but not stderr as we do not guarantee the same exact output? So location info might be different
            let test_case = TestCase::default()
                .with_name(format!(
                    "Run prettified binary from `{}`",
                    original_file.display()
                ))
                .with_binary(binary_name.display())
                .with_exit_code(u8::try_from(code)?);

            Ok(test_case)
        }
    }
}

pub enum AstExport {
    Compile,
    Run,
}

impl Pass for AstExport {
    fn fetch(&self, args: &Args) -> Result<Vec<PathBuf>, Error> {
        let gccrs_path = &args.gccrs_path;
        let tests_path = gccrs_path.join("gcc").join("testsuite").join("rust");
        let output_dir = args.output_dir.join("ast-export");

        // Figure out a nice way to cache things since we don't need to do the copy twice
        // if let AstExport::Run = self {
        //      // The copies are already created in the `Compile` phase
        //      return Ok(fetch_rust_files(&output_dir)
        //          .into_iter()
        //          .map(|entry| entry.path().to_owned())
        //          .collect());
        //  }

        // For each file:
        //      gccrs -frust-dump-ast-pretty <file>
        //      cp gccrs.ast-pretty.dump <new_path>
        let new_files = fetch_rust_files(&tests_path)
            // FIXME: Cannot parallelize this since the AST dump is always the same file...
            // Think about -frust-dump-ast-pretty=<file>?
            .into_iter()
            .map(|entry| {
                let new_path_original = output_dir.join(entry.path());
                let new_path = output_dir.join(entry.path()).with_extension("pretty-rs");

                Compiler::new(Kind::Rust1, args)
                    .command()
                    .arg(entry.path())
                    .arg("-frust-dump-ast-pretty")
                    // No need to go further in the pipeline
                    .arg("-frust-compile-until=lowering")
                    .status()?;

                // Make sure the directory exists
                if let Some(parent) = new_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                fs::copy(entry.path(), new_path_original)?;
                fs::copy("gccrs.ast-pretty.dump", &new_path)?;

                Ok(new_path)
            })
            .collect::<Result<Vec<PathBuf>, Error>>()?;

        Ok(new_files)
    }

    fn adapt(&self, args: &Args, pretty_file: &Path) -> Result<TestCase, Error> {
        match self {
            AstExport::Compile => adapt_compilation(args, pretty_file),
            AstExport::Run => adapt_run(args, pretty_file),
        }
    }
}
