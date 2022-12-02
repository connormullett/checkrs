use crate::Opt;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct Config {
    pub check: bool,
    pub ignore_missing: bool,
    pub quiet: bool,
    pub input_files: Vec<PathBuf>,
}

impl Config {
    pub fn from_opts(opt: &Opt) -> Self {
        Self {
            check: opt.check,
            ignore_missing: opt.ignore_missing,
            quiet: opt.quiet,
            input_files: opt.input_files.clone(),
        }
    }
}
