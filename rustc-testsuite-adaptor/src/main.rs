mod args;
mod error;
mod pass;

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use args::Args;
use error::Error;

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

fn fetch_test_cases(rustc: &Path, out_dir: &Path) -> Vec<Result<PathBuf, Error>> {
    // FIXME: Add more tests than just src/test/ui
    let ui_tests = rustc.join("src").join("test").join("ui");

    WalkDir::new(ui_tests)
        .into_iter()
        // FIXME: We can skip some known test with issues here
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension() == Some(OsStr::new("rs")))
        .map(move |entry| {
            let old_path = entry.path();
            let new_path = old_path.strip_prefix(rustc)?;
            let new_path = out_dir.join(&new_path);

            if let Some(new_parent) = new_path.parent() {
                fs::create_dir_all(new_parent)?;
            }

            fs::copy(old_path, &new_path)?;

            Ok(new_path)
        })
        .collect()
}

fn main() -> anyhow::Result<()> {
    let args = Args::from_args();

    maybe_create_output_dir(&args.output_dir)?;
    if !args.rustc.exists() {
        return Err(Error::NoRustc(args.rustc).into());
    }

    let tuples: Vec<Result<_, Error>> = fetch_test_cases(&args.rustc, &args.output_dir)
        .into_par_iter()
        .map(|path| {
            let path = path?;

            log(format!(
                "running `rustc -Z parse-only --edition 2018 {}`",
                path.display()
            ));

            let is_valid = Command::new("rustc")
                // FIXME: We need to instead build a specific version of rustc to test against rather than using the user's
                .env("RUSTC_BOOTSTRAP", "1")
                .arg("-Z")
                .arg("parse-only")
                .arg("--edition")
                .arg("2021")
                .arg(path.as_os_str())
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .status()?
                .success();

            Ok((path, is_valid))
        })
        .collect();

    let yml: Result<_, Error> =
        tuples
            .into_iter()
            .try_fold(String::from("tests:\n"), |yml, tuple| {
                let (path, is_valid) = tuple?;

                log(format!("generating test case for {}", path.display()));

                let yml = format!("{}  - name: Compile {}\n", yml, &path.display());
                let yml = format!("{}    binary: {}\n", yml, &args.compiler.display());
                // FIXME: Add default timeout instead of hardcoded value
                let yml = format!("{}    timeout: 5\n", yml);
                let yml = format!(
                    "{}    exit_code: {}\n",
                    yml,
                    if is_valid { "0" } else { "1" }
                );
                let yml = format!("{}    args:\n", yml);
                let yml = format!("{}      - \"-fsyntax-only\"\n", yml);
                let yml = format!("{}      - \"{}\"\n", yml, &path.display());

                Ok(yml)
            });

    fs::write(&args.yaml, yml?.as_bytes())?;

    Ok(())
}
