use std::{
    collections::HashSet,
    fmt,
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use tokio::sync::RwLock;

use crate::app::models::Ckey;

#[derive(Debug)]
pub enum Error {
    Read(io::Error),
    Write(io::Error),
}

impl From<Error> for super::Error {
    fn from(value: Error) -> Self {
        Self::Whitelist(value)
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

pub struct WhitelistService {
    whitelist_path: PathBuf,
    whitelist: RwLock<HashSet<String>>,
}

impl WhitelistService {
    pub fn new(whitelist_path: &Path) -> Self {
        Self {
            whitelist_path: whitelist_path.into(),
            whitelist: RwLock::new(HashSet::new()),
        }
    }

    pub async fn load(&self) -> Result<(), Error> {
        let path = self.whitelist_path.clone();
        let whitelist = tokio::task::spawn_blocking(move || {
            let file = File::open(path).map_err(|e| Error::Read(e))?;
            let reader = BufReader::new(file);

            let mut whitelist = HashSet::new();

            for line in reader.lines() {
                let line = line.map_err(|e| Error::Read(e))?;
                if line.starts_with('#') || line.trim().is_empty() {
                    continue;
                }
                whitelist.insert(line.trim().into());
            }

            Ok(whitelist)
        })
        .await
        .expect("Thread panicked")?;

        *self.whitelist.write().await = whitelist;

        Ok(())
    }

    pub async fn store(&self) -> Result<(), Error> {
        let path = self.whitelist_path.clone();
        let whitelist = {
            let guard = self.whitelist.read().await;
            guard.clone()
        };

        tokio::task::spawn_blocking(move || {
            let file = File::create(path).map_err(|e| Error::Write(e))?;
            let mut writer = BufWriter::new(file);

            writeln!(
                writer,
                "# This file is autogenerated\n# Any modifications will be lost"
            )
            .map_err(|e| Error::Write(e))?;

            for ckey in &whitelist {
                writeln!(writer, "{ckey}").map_err(|e| Error::Write(e))?;
            }

            Ok(writer.flush().map_err(|e| Error::Write(e))?)
        })
        .await
        .expect("Thread panicked")
    }

    pub async fn list(&self) -> Vec<String> {
        self.whitelist.read().await.iter().cloned().collect()
    }

    pub async fn insert(&self, ckey: &Ckey) -> Result<bool, Error> {
        let result = {
            let mut guard = self.whitelist.write().await;
            guard.insert(ckey.as_str().to_string())
        };

        if result {
            self.store().await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn remove(&self, ckey: &Ckey) -> Result<bool, Error> {
        let result = {
            let mut guard = self.whitelist.write().await;
            guard.remove(ckey.as_str())
        };

        if result {
            self.store().await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }
}
