use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

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

    #[structopt(parse(from_os_str))]
    input_files: Vec<PathBuf>,
}

struct Config {
    check: bool,
    ignore_missing: bool,
    quiet: bool,
    input_files: Vec<PathBuf>,
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

struct RawChecksum {
    pub data: String,
    pub path: PathBuf,
}

#[derive(Debug)]
struct Checksum {
    hash: String,
    path: PathBuf,
}

impl ToString for Checksum {
    fn to_string(&self) -> String {
        format!("{}  {}", self.hash, self.path.display())
    }
}

enum ChecksumError {
    ImproperFormat,
}

impl ToString for ChecksumError {
    fn to_string(&self) -> String {
        match self {
            ChecksumError::ImproperFormat => {
                "no properly formatted checksum lines found".to_string()
            }
        }
    }
}

fn generate(cfg: &Config) {
    let stdout = io::stdout();
    let mut handle = io::BufWriter::new(stdout);
    for path in &cfg.input_files {
        let mut hasher = Sha256::new();
        match fs::read(path) {
            Ok(data) => {
                hasher.update(data);
                let digest = hasher.finalize();
                let hash = hex::encode(digest);

                let checksum = Checksum {
                    hash,
                    path: path.clone(),
                };

                writeln!(handle, "{}", checksum.to_string()).expect("FIXME");
            }
            Err(e) => {
                print_status(cfg, path, e.to_string());
            }
        }
    }
    handle.flush().expect("FIXME");
}

fn parse_checksum(data: String) -> Result<Checksum, ChecksumError> {
    let mut file_contents = data.trim().split("  ");
    let hash = file_contents
        .next()
        .ok_or(ChecksumError::ImproperFormat)?
        .to_string();
    let path = file_contents
        .next()
        .ok_or(ChecksumError::ImproperFormat)?
        .into();
    Ok(Checksum { path, hash })
}

fn verify_checksum(cfg: &Config, checksum: &Checksum, path: &Path) -> bool {
    match fs::read(&checksum.path) {
        Ok(file_content) => {
            let mut hasher = Sha256::new();
            hasher.update(file_content);
            let digest = hasher.finalize();
            let hex = hex::encode(digest);

            let status = if hex == checksum.hash { "OK" } else { "FAILED" };
            print_status(cfg, path, status.to_string());
            hex == checksum.hash
        }
        Err(e) => {
            if !cfg.ignore_missing {
                print_status(cfg, path, e.to_string());
            }
            false
        }
    }
}

fn verify(cfg: &Config) {
    cfg.input_files
        .iter()
        .filter_map(|path| {
            fs::read_to_string(path)
                .map_err(|e| print_status(cfg, path, e.to_string()))
                .ok()
                .map(|data| RawChecksum {
                    data,
                    path: path.clone(),
                })
        })
        .filter_map(|raw_checksum| {
            parse_checksum(raw_checksum.data)
                .map_err(|e| print_status(cfg, &raw_checksum.path, e.to_string()))
                .ok()
        })
        .for_each(|ref checksum| {
            verify_checksum(cfg, checksum, &checksum.path);
        })
}

fn print_status(cfg: &Config, path: &Path, msg: String) {
    if !cfg.quiet {
        println!("{}: {}", path.display(), msg);
    }
}

fn main() {
    let opt = Opt::from_args();

    let cfg = Config::from_opts(&opt);

    if cfg.check {
        verify(&cfg)
    } else {
        generate(&cfg)
    };
}
