use crate::passes::PassKind;

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[arg(
        short,
        long,
        help = "output directory which will contain the new tests"
    )]
    pub(crate) output_dir: PathBuf,
    #[arg(short, long, help = "generated YAML ftf file name")]
    pub(crate) yaml: PathBuf,
    #[arg(short, long, help = "path to the rustc compiler to use")]
    pub(crate) rustc: PathBuf,
    #[arg(short, long, help = "path to the gccrs compiler to use")]
    pub(crate) gccrs: PathBuf,
    #[arg(long, help = "path to a cloned rust repository")]
    pub(crate) rust_path: PathBuf,
    #[arg(long, help = "path to a cloned gccrs repository")]
    pub(crate) gccrs_path: PathBuf,
    #[arg(short, long, help = "pass to to run in the adaptor")]
    pub(crate) pass: PassKind,
    #[arg(short, long, help = "amount of threads to use", default_value = "1")]
    pub(crate) jobs: usize,
}
