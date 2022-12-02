#![allow(dead_code)]

use crate::Config;
use std::cell::RefCell;
use std::fs;
use std::io::stderr;
use std::io::Write;
use std::rc::Rc;

use crate::checksum::RawChecksum;
use crate::{parse_checksum, verify_checksum};

#[derive(Debug, Default)]
pub struct Verifier {
    config: Config,
    failures: u32,
    status_code: u8,
}

impl Verifier {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    pub fn verify(&self) {
        // TODO: maybe move this handle to be a member?
        let stderr = Rc::new(RefCell::new(stderr()));

        self.config
            .input_files
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
                        if !self.config.quiet {
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
                verify_checksum(&self.config, checksum, &stderr);
            });

        stderr.borrow_mut().flush().expect("FIXME");
    }

    fn report(&self) {
        todo!()
    }
}
