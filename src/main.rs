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
            ChecksumError::ImproperFormat => format!("no properly formatted checksum lines found"),
        }
    }
}

fn read_file(path: &PathBuf) -> io::Result<String> {
    Ok(fs::read_to_string(path)?)
}

fn generate(cfg: &Config) {
    for path in &cfg.input_files {
        let mut hasher = Sha256::new();
        match read_file(&path) {
            Ok(data) => {
                hasher.update(data);
                let digest = hasher.finalize();
                let hash = hex::encode(digest);

                let checksum = Checksum {
                    hash,
                    path: path.clone(),
                };

                println!("{}", checksum.to_string());
            }
            Err(e) => print_status(cfg, &path, e.to_string()),
        }
    }
}

fn parse_checksum(data: String) -> Result<Checksum, ChecksumError> {
    // FIXME: File names containing double spaces might mess this up
    let mut file_contents: Vec<&str> = data.trim().split("  ").collect();
    if file_contents.is_empty() || file_contents.len() > 2 {
        return Err(ChecksumError::ImproperFormat);
    } else {
        let path = match file_contents.pop() {
            Some(path) => PathBuf::from(path),
            None => return Err(ChecksumError::ImproperFormat),
        };

        let hash = match file_contents.pop() {
            Some(data) => data.to_string(),
            None => return Err(ChecksumError::ImproperFormat),
        };

        Ok(Checksum { path, hash })
    }
}

fn verify_checksum(cfg: &Config, checksum: Checksum, path: &PathBuf) -> bool {
    match read_file(&checksum.path) {
        Ok(file_content) => {
            let mut hasher = Sha256::new();
            hasher.update(file_content);
            let digest = hasher.finalize();
            let hex = hex::encode(digest);

            let status = if hex == checksum.hash { "OK" } else { "FAILED" };
            print_status(cfg, &path, status.to_string());
            hex == checksum.hash
        }
        Err(e) => {
            if !cfg.ignore_missing {
                print_status(cfg, &path, e.to_string());
            }
            false
        }
    }
}

fn verify(cfg: &Config) {
    let mut warnings = 0;
    for path in &cfg.input_files {
        match read_file(path) {
            Ok(data) => match parse_checksum(data) {
                Ok(checksum) => {
                    if !verify_checksum(cfg, checksum, &path) {
                        warnings += 1;
                    }
                }
                Err(e) => {
                    if cfg.warn {
                        print_status(cfg, &path, e.to_string());
                    }
                }
            },
            Err(e) => {
                if !cfg.ignore_missing {
                    print_status(cfg, &path, e.to_string());
                }
            }
        };
    }

    if warnings > 0 {
        println!("WARNING: {} computed checksum did NOT match", warnings);
    }
}

fn print_status(cfg: &Config, path: &PathBuf, msg: String) {
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
    }
}
