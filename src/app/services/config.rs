use std::{
    fmt,
    fs::File,
    io::{self, BufReader},
    path::{Path, PathBuf},
};

use ron::error::SpannedError;
use tokio::sync::{RwLock, RwLockReadGuard};

use crate::AppConfig;

#[derive(Debug)]
pub enum Error {
    Read(SpannedError),
    Write(ron::Error),
}

impl From<Error> for super::Error {
    fn from(value: Error) -> Self {
        Self::Config(value)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::Read(ref e) => write!(f, "read: {}", e),
            Error::Write(ref e) => write!(f, "write: {}", e),
        }
    }
}

pub struct ConfigService {
    config_path: PathBuf,
    pub config: RwLock<AppConfig>,
}

impl ConfigService {
    pub fn new(config: AppConfig, config_path: &Path) -> Self {
        Self {
            config_path: config_path.into(),
            config: RwLock::new(config),
        }
    }

    pub async fn load(&self) -> Result<(), Error> {
        let path = self.config_path.clone();
        let config = tokio::task::spawn_blocking(move || {
            let file = match File::open(path) {
                Ok(file) => file,
                Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
                Err(e) => return Err(Error::Read(e.into())),
            };
            let reader = BufReader::new(file);

            Ok(Some(
                ron::de::from_reader(reader).map_err(|e| Error::Read(e))?,
            ))
        })
        .await
        .expect("Thread panicked")?;

        match config {
            Some(config) => *self.config.write().await = config,
            None => (),
        };

        Ok(())
    }

    pub async fn store(&self) -> Result<(), Error> {
        let path = self.config_path.clone();
        let value = {
            let guard = self.config.read().await;
            guard.clone()
        };

        tokio::task::spawn_blocking(move || value.store(&path))
            .await
            .expect("Thread panicked")
            .map_err(|e| Error::Write(e))
    }

    pub async fn get(&self) -> RwLockReadGuard<'_, AppConfig> {
        self.config.read().await
    }
}
