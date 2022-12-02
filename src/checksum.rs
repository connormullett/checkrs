use std::fmt;
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

impl TryFrom<String> for Checksum {
    type Error = ChecksumError;

    fn try_from(data: String) -> Result<Self, Self::Error> {
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
}

impl ToString for Checksum {
    fn to_string(&self) -> String {
        format!("{}  {}", self.hash, self.path.display())
    }
}

#[derive(Clone, Debug)]
pub enum ChecksumError {
    ImproperFormat(String),
}

impl ChecksumError {
    pub fn inner(&self) -> &String {
        match self {
            ChecksumError::ImproperFormat(inner) => inner,
        }
    }
}

impl fmt::Display for ChecksumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invalid checksum format. Affected line had: {}",
            self.inner()
        )
    }
}
