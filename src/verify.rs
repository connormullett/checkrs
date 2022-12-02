#![allow(dead_code)]

use crate::Config;
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::fs;
use std::io::stderr;
use std::io::stdout;
use std::io::Stderr;
use std::io::Stdout;
use std::io::Write;
use std::rc::Rc;

use crate::checksum::{Checksum, RawChecksum};

#[derive(Debug)]
pub struct Verifier {
    config: Config,
    failures: Vec<Checksum>,
    status_code: u8,
    error_handle: Rc<RefCell<Stderr>>,
    output_handle: Rc<RefCell<Stdout>>,
}

impl Default for Verifier {
    fn default() -> Self {
        Self {
            config: Default::default(),
            failures: Default::default(),
            status_code: Default::default(),
            error_handle: Rc::new(RefCell::new(stderr())),
            output_handle: Rc::new(RefCell::new(stdout())),
        }
    }
}

impl Verifier {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    pub fn verify(&mut self) {
        self.config
            .input_files
            .iter()
            .filter_map(|path| {
                fs::read_to_string(path)
                    .map_err(|e| {
                        writeln!(self.error_handle.borrow_mut(), "{}: {}", path.display(), e)
                    })
                    .ok()
                    .map(|data| RawChecksum {
                        data,
                        path: path.clone(),
                    })
            })
            .filter_map(|raw_checksum| {
                Checksum::try_from(raw_checksum.data)
                    .map_err(|e| {
                        if !self.config.quiet {
                            writeln!(
                                self.error_handle.borrow_mut(),
                                "{}: {}",
                                raw_checksum.path.display(),
                                e
                            )
                            .unwrap();
                        }
                    })
                    .ok()
            })
            .for_each(|ref checksum| {
                self.verify_checksum(checksum);
            });

        self.error_handle.borrow_mut().flush().unwrap();
        self.output_handle.borrow_mut().flush().unwrap();
    }

    fn verify_checksum(&self, checksum: &Checksum) {
        match fs::read(&checksum.path) {
            Ok(file_content) => {
                let mut hasher = Sha256::new();
                hasher.update(file_content);
                let digest = hasher.finalize();
                let hex = hex::encode(digest);

                if hex == checksum.hash {
                    writeln!(
                        self.output_handle.borrow_mut(),
                        "OK: {}",
                        checksum.path.display()
                    )
                    .unwrap();
                } else {
                    writeln!(
                        self.error_handle.borrow_mut(),
                        "FAILED: {}",
                        checksum.path.display(),
                    )
                    .unwrap();
                }
            }
            Err(e) => {
                if !self.config.ignore_missing {
                    writeln!(
                        self.error_handle.borrow_mut(),
                        "{}: {}",
                        checksum.path.display(),
                        e
                    )
                    .unwrap();
                }
            }
        }
    }
}
