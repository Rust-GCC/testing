mod adaptor;
mod args;
mod error;

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use adaptor::FileAdaptorExt;
use args::Args;
use error::Error;

use structopt::StructOpt;
use walkdir::WalkDir;

fn maybe_create_output_dir(path: &Path) -> Result<(), Error> {
    match path.exists() {
        true => Ok(()),
        false => Ok(fs::create_dir(path)?),
    }
}

fn fetch_test_cases<'i>(
    rustc: &'i Path,
    out_dir: &'i Path,
) -> impl Iterator<Item = Result<PathBuf, Error>> + 'i {
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
}

fn main() -> anyhow::Result<()> {
    let args = Args::from_args();

    maybe_create_output_dir(&args.output_dir)?;
    if !args.rustc.exists() {
        return Err(Error::NoRustc(args.rustc).into());
    }

    let yml: Result<String, Error> = fetch_test_cases(&args.rustc, &args.output_dir)
        .adapt()
        .try_fold(String::from("tests:\n"), |yml, path| {
            let path = path?;

            let yml = format!("{}  - name: Compile {}\n", yml, &path.display());
            let yml = format!("{}    binary: {}\n", yml, &args.compiler.display());
            let yml = format!("{}    timeout: 5\n", yml);
            let yml = format!("{}    args:\n", yml);
            let yml = format!("{}      - \"{}\"\n", yml, &path.display());

            Ok(yml)
        });

    fs::write(&args.yaml, yml?.as_bytes())?;

    Ok(())
}
