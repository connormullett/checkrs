#![allow(dead_code)]

use crate::checksum::ChecksumError;
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

use crate::checksum::Checksum;

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
                    .map_err(|e| self.write_error(e.into()))
                    .ok()
            })
            .for_each(|checksum| {
                checksum
                    .lines()
                    .filter_map(|sum| {
                        Checksum::try_from(sum.to_string())
                            .map_err(|e| self.write_error(e))
                            .ok()
                    })
                    .for_each(|sum| self.verify_checksum(sum))
            });

        self.flush();
    }

    fn write_error(&self, error: ChecksumError) -> ChecksumError {
        writeln!(self.error_handle.borrow_mut(), "Error: {}", error).unwrap();
        error
    }

    fn verify_checksum(&self, checksum: Checksum) {
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

    fn flush(&mut self) {
        self.error_handle.borrow_mut().flush().unwrap();
        self.output_handle.borrow_mut().flush().unwrap();
    }
}
