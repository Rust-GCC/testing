mod args;
mod error;
mod pass;

use std::ffi::OsStr;
use std::fs;
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

        WalkDir::new(ui_tests)
            .into_iter()
            // FIXME: We can skip some known test with issues here
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension() == Some(OsStr::new("rs")))
            .map(move |entry| {
                let old_path = entry.path();
                let new_path = old_path.strip_prefix(&rust_path)?;
                let new_path = args.output_dir.join(&new_path);

                if let Some(new_parent) = new_path.parent() {
                    fs::create_dir_all(new_parent)?;
                }

                fs::copy(old_path, &new_path)?;

                Ok(new_path)
            })
            .collect()
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
    fn fetch(&self, _args: &Args) -> Result<Vec<PathBuf>, Error> {
        todo!()
    }

    fn adapt(&self, _args: &Args, _file: &Path) -> Result<TestCase, Error> {
        todo!()
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

    let yml_header = String::from("tests:\n");

    let yml: Result<Vec<String>, Error> = args
        .passes
        .iter()
        .map(|pass| {
            let pass = pass_dispatch(pass);
            let files = pass.fetch(&args)?;

            // FIXME: Use returned string
            let yml = apply_pass(&*pass, &args, &files)?;

            Ok(yml)
        })
        .collect();

    fs::write(&args.yaml, yml_header.as_bytes())?;
    yml?.iter()
        .try_for_each(|yml| fs::write(&args.yaml, yml.as_bytes()))?;

    Ok(())
}
