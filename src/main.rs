use std::fs;
use std::io::stderr;
use std::io::{stdout, Write};
use std::path::PathBuf;

use sha2::{Digest, Sha256};
use structopt::StructOpt;
use verify::Verifier;

mod checksum;
mod config;
mod verify;

use checksum::Checksum;
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

fn main() {
    let opt = Opt::from_args();

    let cfg = Config::from_opts(&opt);

    if cfg.check {
        let mut verifier = Verifier::new(cfg);
        verifier.verify();
    } else {
        generate(&cfg)
    };
}
