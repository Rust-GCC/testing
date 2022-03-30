mod args;
mod error;
mod pass;

use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use args::Args;
use error::Error;
use pass::{Pass, PassKind, TestCase};

use rayon::prelude::*;
use structopt::StructOpt;
use walkdir::WalkDir;

fn log<T: Into<String>>(msg: T) {
    eprintln!("[log] {}", msg.into());
}

fn maybe_create_output_dir(path: &Path) -> Result<(), Error> {
    match path.exists() {
        true => Ok(()),
        false => Ok(fs::create_dir(path)?),
    }
}

fn copy_rs_files(from: &Path, to: &Path, prefix_to_strip: &Path) -> Result<Vec<PathBuf>, Error> {
    WalkDir::new(from)
        .into_iter()
        // FIXME: We can skip some known test with issues here
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension() == Some(OsStr::new("rs")))
        .map(move |entry| {
            let old_path = entry.path();
            let new_path = old_path.strip_prefix(&prefix_to_strip)?;
            let new_path = to.join(&new_path);

            if let Some(new_parent) = new_path.parent() {
                fs::create_dir_all(new_parent)?;
            }

            fs::copy(old_path, &new_path)?;

            Ok(new_path)
        })
        .collect()
}

struct GccrsParsing;

impl Pass for GccrsParsing {
    fn fetch(&self, args: &Args) -> Result<Vec<PathBuf>, Error> {
        // FIXME: Add more tests than just src/test/ui
        let rust_path = &args.rust_path;
        let ui_tests = rust_path.join("src").join("test").join("ui");

        log(format!(
            "fetching test cases for gccrs-parsing from `{}`",
            ui_tests.display()
        ));

        copy_rs_files(&ui_tests, &args.output_dir, rust_path)
    }

    fn adapt(&self, args: &Args, file: &Path) -> Result<TestCase, Error> {
        log(format!(
            "running `rustc -Z parse-only --edition 2018 {}`",
            file.display()
        ));

        let is_valid = Command::new(&args.rustc)
            // FIXME: We need to instead build a specific version of rustc to test against rather than using the user's
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

unsafe impl Sync for GccrsParsing {}

struct RustcDejagnu;

impl Pass for RustcDejagnu {
    fn fetch(&self, args: &Args) -> Result<Vec<PathBuf>, Error> {
        let gccrs_path = &args.gccrs_path;
        let tests_path = gccrs_path.join("gcc").join("testsuite").join("rust");

        log(format!(
            "fetching test cases for rustc-dejagnu from `{}`",
            gccrs_path.display()
        ));

        copy_rs_files(&tests_path, &args.output_dir, gccrs_path)
    }

    fn adapt(&self, args: &Args, file: &Path) -> Result<TestCase, Error> {
        log(format!("adapting gccrs test case `{}`", file.display()));

        let test_content = fs::read_to_string(file)?;

        let exit_code = if test_content.contains("dg-error") {
            1
        } else {
            0
        };

        // FIXME: This should be removed once we have a proper main shim in gccrs
        // This is to make sure that we can't ever get a "success" because a test
        // contains a dg-error directive and a `fn main() -> i32` so rustc produces
        // the correct exit code
        let exit_code = if test_content.contains("fn main() -> i32") {
            255
        } else {
            exit_code
        };

        let test_case = TestCase::default()
            .with_name(format!("Run rustc on `{}`", file.display()))
            .with_binary(args.rustc.display())
            .with_exit_code(exit_code)
            .with_timeout(5)
            .with_arg(file.display())
            .with_arg("-o") // Compile all files to the same executable name to avoid having to clean up 500 executables...
            .with_arg("rustc_out");

        Ok(test_case)
    }
}

unsafe impl Sync for RustcDejagnu {}

fn pass_dispatch(pass: &PassKind) -> Box<dyn Pass> {
    match pass {
        PassKind::GccrsParsing => Box::new(GccrsParsing),
        PassKind::RustcDejagnu => Box::new(RustcDejagnu),
    }
}

fn apply_pass(pass: &dyn Pass, args: &Args, files: &[PathBuf]) -> Result<String, Error> {
    files
        .into_par_iter()
        .map(|file| pass.adapt(args, file))
        .try_fold(String::new, |acc, test_case: Result<_, Error>| {
            Ok(format!("{}\n{}", acc, test_case?))
        })
        .collect()
}

fn main() -> anyhow::Result<()> {
    let args = Args::from_args();
    maybe_create_output_dir(&args.output_dir)?;
    if !args.rust_path.exists() {
        return Err(Error::NoRust(args.rust_path).into());
    }
    if !args.gccrs_path.exists() {
        return Err(Error::NoGccrs(args.gccrs_path).into());
    }

    let ftf_header = String::from("tests:\n");

    let test_suites: Result<Vec<String>, Error> = args
        .passes
        .iter()
        .map(|pass| {
            let pass = pass_dispatch(pass);
            let files = pass.fetch(&args)?;

            let test_suite = apply_pass(&*pass, &args, &files)?;

            Ok(test_suite)
        })
        .collect();

    let mut yml = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&args.yaml)?;

    yml.write_all(ftf_header.as_bytes())?;

    test_suites?
        .iter()
        .try_for_each(|suite| yml.write_all(suite.as_bytes()))?;

    Ok(())
}
