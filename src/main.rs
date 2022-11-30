use std::cell::RefCell;
use std::fs;
use std::io::stderr;
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::rc::Rc;

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

    /// The files to be hashed and verified
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
    ImproperFormat(String),
}

impl ToString for ChecksumError {
    fn to_string(&self) -> String {
        match self {
            ChecksumError::ImproperFormat(msg) => msg.to_owned(),
        }
    }
}

fn generate(cfg: &Config) {
    
    if cfg.quiet {
        println!("Checkrs: The quiet option is meaningful only when verifying checksums");
        println!("Try 'checkrs --help' for more information");
        return
    }

    let mut stdout = stdout();
    let mut stderr = stderr();

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

                writeln!(stdout, "{}", checksum.to_string()).expect("FIXME");
            }
            Err(e) => {
                writeln!(stderr, "{}: {}", path.display(), e).expect("FIXME");
            }
        }
    }

    stdout.flush().expect("FIXME");
    stderr.flush().expect("FIXME");
}

fn parse_checksum(data: String) -> Result<Checksum, ChecksumError> {
    let mut file_contents = data.trim().split("  ");
    let hash = file_contents
        .next()
        .ok_or_else(|| {
            ChecksumError::ImproperFormat(format!(
                "Invalid checksum format. Affected line had: {}",
                data
            ))
        })?
        .to_string();
    let path = file_contents
        .next()
        .ok_or_else(|| {
            ChecksumError::ImproperFormat(format!(
                "Invalid checksum format. Affected line had: {}",
                data
            ))
        })?
        .into();
    Ok(Checksum { path, hash })
}

fn verify_checksum<W: Write>(cfg: &Config, checksum: &Checksum, handle: &Rc<RefCell<W>>) -> bool {
    match fs::read(&checksum.path) {
        Ok(file_content) => {
            let mut hasher = Sha256::new();
            hasher.update(file_content);
            let digest = hasher.finalize();
            let hex = hex::encode(digest);

            let status = if hex == checksum.hash { "OK" } else { "FAILED" };
            writeln!(
                handle.borrow_mut(),
                "{}: {}",
                checksum.path.display(),
                status
            )
            .expect("FIXME");
            hex == checksum.hash
        }
        Err(e) => {
            if !cfg.ignore_missing {
                writeln!(handle.borrow_mut(), "{}: {}", checksum.path.display(), e).expect("FIXME");
            }
            false
        }
    }
}

fn verify(cfg: &Config) {
    let stderr = Rc::new(RefCell::new(stderr()));

    cfg.input_files
        .iter()
        .filter_map(|path| {
            fs::read_to_string(path)
                .map_err(|e| writeln!(stderr.borrow_mut(), "{}: {}", path.display(), e))
                .ok()
                .map(|data| RawChecksum {
                    data,
                    path: path.clone(),
                })
        })
        .filter_map(|raw_checksum| {
            parse_checksum(raw_checksum.data)
                .map_err(|e| {
                    if !cfg.quiet {
                        writeln!(
                            stderr.borrow_mut(),
                            "{}: {}",
                            raw_checksum.path.display(),
                            e.to_string()
                        )
                        .expect("FIXME");
                    }
                })
                .ok()
        })
        .for_each(|ref checksum| {
            verify_checksum(cfg, checksum, &stderr);
        });

    stderr.borrow_mut().flush().expect("FIXME");
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
