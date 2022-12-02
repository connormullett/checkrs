use std::cell::RefCell;
use std::fs;
use std::io::stderr;
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::rc::Rc;

use sha2::{Digest, Sha256};
use structopt::StructOpt;
use verify::Verifier;

mod checksum;
mod config;
mod verify;

use checksum::{Checksum, ChecksumError};
use config::Config;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "checkrs",
    about = "Checksum generation and verification tool in Rust"
)]
pub struct Opt {
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

fn generate(cfg: &Config) {
    if cfg.quiet {
        println!("Checkrs: The quiet option is meaningful only when verifying checksums");
        println!("Try 'checkrs --help' for more information");
        return;
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

fn main() {
    let opt = Opt::from_args();

    let cfg = Config::from_opts(&opt);

    if cfg.check {
        let verifier = Verifier::new(cfg);
        verifier.verify();
    } else {
        generate(&cfg)
    };
}
