use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct Opts {
    /// presistent sqlite path, if the param is not passed, use memory sqlite as default
    #[clap(short, long)]
    pub db: Option<PathBuf>,
}
