use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone)]
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

impl From<std::io::Error> for ChecksumError {
    fn from(error: std::io::Error) -> Self {
        Self::ImproperFormat(error.to_string())
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
