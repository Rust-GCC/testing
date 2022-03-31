mod args;
mod error;
mod passes;

use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use args::Args;
use error::Error;
use passes::{Pass, PassKind};

use rayon::prelude::*;
use structopt::StructOpt;
use walkdir::WalkDir;

pub fn log<T: Into<String>>(msg: T) {
    eprintln!("[log] {}", msg.into());
}

fn maybe_create_output_dir(path: &Path) -> Result<(), Error> {
    match path.exists() {
        true => Ok(()),
        false => Ok(fs::create_dir(path)?),
    }
}

pub fn copy_rs_files(
    from: &Path,
    to: &Path,
    prefix_to_strip: &Path,
) -> Result<Vec<PathBuf>, Error> {
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

fn pass_dispatch(pass: &PassKind) -> Box<dyn Pass> {
    match pass {
        PassKind::GccrsParsing => Box::new(passes::GccrsParsing),
        PassKind::RustcDejagnu => Box::new(passes::RustcDejagnu),
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
        .truncate(true)
        .write(true)
        .open(&args.yaml)?;

    yml.write_all(ftf_header.as_bytes())?;

    test_suites?
        .iter()
        .try_for_each(|suite| yml.write_all(suite.as_bytes()))?;

    Ok(())
}
