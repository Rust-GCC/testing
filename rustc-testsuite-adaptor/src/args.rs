use crate::passes::PassKind;

use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Args {
    #[structopt(
        short,
        long,
        help = "output directory which will contain the new tests"
    )]
    pub(crate) output_dir: PathBuf,
    #[structopt(short, long, help = "generated YAML ftf file name")]
    pub(crate) yaml: PathBuf,
    #[structopt(short, long, help = "path to the rustc compiler to use")]
    pub(crate) rustc: PathBuf,
    #[structopt(short, long, help = "path to the gccrs/rust1 compiler to use")]
    pub(crate) gccrs: PathBuf,
    #[structopt(long, help = "path to a cloned rust repository")]
    pub(crate) rust_path: PathBuf,
    #[structopt(long, help = "path to a cloned gccrs repository")]
    pub(crate) gccrs_path: PathBuf,
    #[structopt(short, long, help = "passes to to run in the adaptor")]
    pub(crate) passes: Vec<PassKind>,
}
