mod args;
mod error;

use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use args::Args;
use error::Error;

use structopt::StructOpt;
use walkdir::WalkDir;

macro_rules! log {
    (info $($msg:expr),*) => {{
        use std::io::Write;

        eprint!("[INFO] ");
        eprint!($($msg),*);

        std::io::stderr().lock().flush().unwrap();
    }};
    ($($msg:expr),*) => {
        log!(info $($msg),*)
    };
}

fn maybe_create_output_dir(path: &Path) -> Result<(), Error> {
    match path.exists() {
        true => Ok(()),
        false => Ok(fs::create_dir(path)?),
    }
}

fn fetch_test_cases(rustc: &Path, out_dir: &Path) -> Result<(), Error> {
    // FIXME: Add more tests than just src/test/ui
    let ui_tests = rustc.join("src").join("test").join("ui");

    for entry in WalkDir::new(ui_tests) {
        let entry = entry?;
        let old_path = entry.path();

        if old_path.extension() != Some(OsStr::new("rs")) {
            continue;
        }

        let new_path = old_path.strip_prefix(rustc)?;
        let new_path = out_dir.join(&new_path);

        if let Some(new_parent) = new_path.parent() {
            fs::create_dir_all(new_parent)?;
        }

        log!(
            info
            "copying file `{}` -> `{}`\r",
            &old_path.display(),
            &new_path.display()
        );

        fs::copy(old_path, new_path)?;
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::from_args();

    maybe_create_output_dir(&args.output_dir)?;
    if !args.rustc.exists() {
        return Err(Error::NoRustc(args.rustc).into());
    }

    fetch_test_cases(&args.rustc, &args.output_dir)?;

    Ok(())
}
