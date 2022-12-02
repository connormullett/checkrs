#![allow(dead_code)]

use crate::Config;
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::fs;
use std::io::stderr;
use std::io::Stderr;
use std::io::Write;
use std::rc::Rc;

use crate::checksum::{Checksum, RawChecksum};

#[derive(Debug)]
pub struct Verifier {
    config: Config,
    failures: Vec<Checksum>,
    status_code: u8,
    error_handle: Rc<RefCell<Stderr>>,
}

impl Default for Verifier {
    fn default() -> Self {
        Self {
            config: Default::default(),
            failures: Default::default(),
            status_code: Default::default(),
            error_handle: Rc::new(RefCell::new(stderr())),
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
                                e.to_string()
                            )
                            .expect("FIXME");
                        }
                    })
                    .ok()
            })
            .for_each(|ref checksum| {
                self.verify_checksum(checksum);
            });

        self.error_handle.borrow_mut().flush().expect("FIXME");
    }

    fn verify_checksum(&self, checksum: &Checksum) -> bool {
        match fs::read(&checksum.path) {
            Ok(file_content) => {
                let mut hasher = Sha256::new();
                hasher.update(file_content);
                let digest = hasher.finalize();
                let hex = hex::encode(digest);

                let status = if hex == checksum.hash { "OK" } else { "FAILED" };
                writeln!(
                    self.error_handle.borrow_mut(),
                    "{}: {}",
                    checksum.path.display(),
                    status
                )
                .expect("FIXME");
                hex == checksum.hash
            }
            Err(e) => {
                if !self.config.ignore_missing {
                    writeln!(
                        self.error_handle.borrow_mut(),
                        "{}: {}",
                        checksum.path.display(),
                        e
                    )
                    .expect("FIXME");
                }
                false
            }
        }
    }

    fn report(&self) {
        todo!()
    }
}
