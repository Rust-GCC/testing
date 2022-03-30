use crate::pass::PassKind;

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
    #[structopt(short, long, help = "path to the rustc repository")]
    pub(crate) rustc: PathBuf,
    #[structopt(short, long, help = "path to the compiler to use")]
    pub(crate) compiler: PathBuf,
    #[structopt(short, long, help = "passes to to run in the adaptor")]
    pub(crate) passes: Vec<PassKind>,
}
