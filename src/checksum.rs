use std::path::PathBuf;

pub struct RawChecksum {
    pub data: String,
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct Checksum {
    pub hash: String,
    pub path: PathBuf,
}

impl ToString for Checksum {
    fn to_string(&self) -> String {
        format!("{}  {}", self.hash, self.path.display())
    }
}

pub enum ChecksumError {
    ImproperFormat(String),
}

impl ToString for ChecksumError {
    fn to_string(&self) -> String {
        match self {
            ChecksumError::ImproperFormat(msg) => msg.to_owned(),
        }
    }
}
