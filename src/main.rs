use std::{fs, io, path::PathBuf};

use sha2::{Digest, Sha256};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "checkrs",
    about = "Checksum generation and verification tool in Rust"
)]
struct Opt {
    /// read checksum from the FILEs and check them
    #[structopt(short, long)]
    check: bool,

    /// don't fail or report status for missing files
    #[structopt(short, long)]
    ignore_missing: bool,

    /// don't print OK for each successfully verified file
    #[structopt(short, long)]
    quiet: bool,

    /// exit non-zero for improperly formated checksum lines
    #[structopt(short, long)]
    strict: bool,

    /// warn about improperly formatted checksum lines
    #[structopt(short, long)]
    warn: bool,

    #[structopt(parse(from_os_str))]
    input_files: Vec<PathBuf>,
}

struct Config {
    check: bool,
    ignore_missing: bool,
    quiet: bool,
    strict: bool,
    warn: bool,
    input_files: Vec<PathBuf>,
}

impl Config {
    pub fn from_opts(opt: &Opt) -> Self {
        Self {
            check: opt.check,
            ignore_missing: opt.ignore_missing,
            quiet: opt.quiet,
            strict: opt.strict,
            warn: opt.warn,
            input_files: opt.input_files.clone(),
        }
    }
}

struct Checksum {
    hash: String,
    path: PathBuf,
}

impl ToString for Checksum {
    fn to_string(&self) -> String {
        format!("{}  {}", self.hash, self.path.display())
    }
}

fn read_file(path: &PathBuf) -> io::Result<String> {
    Ok(fs::read_to_string(path)?)
}

fn generate(paths: Vec<PathBuf>) {
    for path in paths {
        let mut hasher = Sha256::new();
        if let Ok(data) = read_file(&path) {
            hasher.update(data);
            let digest = hasher.finalize();
            let hash = hex::encode(digest);
            let checksum = Checksum { hash, path };
            println!("{}", checksum.to_string());
        }
    }
}

fn check() {}

fn main() {
    let opt = Opt::from_args();

    let cfg = Config::from_opts(&opt);

    if cfg.check {
        check()
    } else {
        generate(cfg.input_files)
    }
}
