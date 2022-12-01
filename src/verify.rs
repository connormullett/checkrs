#![allow(dead_code)]

use crate::Config;

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
        todo!()
    }

    fn report(&self) {
        todo!()
    }
}
