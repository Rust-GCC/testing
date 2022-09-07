mod args;
mod error;
mod log;
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
use which::which;

fn maybe_create_output_dir(path: &Path) -> Result<(), Error> {
    if !path.exists() {
        fs::create_dir(path)?;
    }
    Ok(())
}

/// Copies `*.rs` files from the path `from` to the path `to`, while stripping the prefix
/// `prefix_to_strip` from the path.
///
/// # Errors
///
/// This functions returns an error if either
/// - it fails to strip the prefix
/// - it fails to create the new directory
/// - it fails to copy the file to the new location
pub fn copy_rs_files(
    from: &Path,
    to: &Path,
    prefix_to_strip: &Path,
) -> Result<Vec<PathBuf>, Error> {
    WalkDir::new(from)
        .into_iter()
        // FIXME: We can skip some known test with issues here
        .filter_map(Result::ok)
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

fn pass_dispatch(pass: &PassKind) -> Vec<Box<dyn Pass>> {
    match pass {
        PassKind::GccrsParsing => vec![Box::new(passes::GccrsParsing)],
        PassKind::RustcDejagnu => vec![Box::new(passes::RustcDejagnu)],
        PassKind::GccrsRustcSucess => vec![Box::new(passes::GccrsRustcSuccesses::Full)],
        PassKind::GccrsRustcSucessNoStd => vec![Box::new(passes::GccrsRustcSuccesses::NoStd)],
        PassKind::GccrsRustcSucessNoCore => vec![Box::new(passes::GccrsRustcSuccesses::NoCore)],
        PassKind::Blake3 => passes::Blake3::variants()
            .into_iter()
            .map(|blake_variant| Box::new(blake_variant) as Box<dyn Pass>)
            .collect(),
        PassKind::LibCore149 => vec![Box::new(passes::LibCore::V149)],
    }
}

fn apply_pass(pass: &dyn Pass, args: &Args, files: &[PathBuf]) -> Result<String, Error> {
    files
        .into_par_iter()
        .map(|file| pass.adapt(args, file))
        .try_fold(String::new, |acc, test_case: Result<_, Error>| {
            Ok(format!("{}{}", acc, test_case?))
        })
        .collect()
}

fn warn_on_file_not_found(name: &str, path: &Path) {
    if which(path).is_err() {
        warn!(
            "given path to {} ({}) does not point to a valid file. \
            If you're not trying to run the test suite from the current directory, \
            you can ignore this warning",
            name,
            path.display()
        );
    }
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

    warn_on_file_not_found("rustc", &args.rustc);
    warn_on_file_not_found("gccrs", &args.gccrs);

    let ftf_header = String::from("tests:\n");
    rayon::ThreadPoolBuilder::new()
        .num_threads(args.jobs)
        .build_global()?;

    let test_suites: Result<Vec<String>, Error> = args
        .passes
        .par_iter()
        .map(|pass_kind| {
            log!("running pass `{}`...", pass_kind);

            let passes = pass_dispatch(pass_kind);

            passes
                .iter()
                .map(|pass| {
                    log!("fetching test files for `{}`...", pass_kind);

                    let files = pass.fetch(&args)?;

                    log!(
                        "generating test cases for `{}`... this might take a while",
                        pass_kind
                    );

                    // This is ugly!
                    let test_suite = apply_pass(&**pass, &args, &files)?;

                    log!("`{}` pass complete!", pass_kind);

                    Ok(test_suite)
                })
                .collect()
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
