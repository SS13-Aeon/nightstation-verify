use std::{
    collections::HashMap,
    fmt,
    fs::File,
    io::{self, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use ron::{error::SpannedError, ser::PrettyConfig};
use serenity::model::prelude::UserId;
use tokio::sync::RwLock;

use crate::app::models::Ckey;

#[derive(Debug)]
pub enum Error {
    Read(SpannedError),
    Write(ron::Error),
}

impl From<Error> for super::Error {
    fn from(value: Error) -> Self {
        Self::Ckey(value)
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

pub struct CkeyService {
    path: PathBuf,
    ckeys: RwLock<HashMap<UserId, String>>,
}

impl CkeyService {
    pub fn new(data_path: &Path) -> Self {
        Self {
            path: data_path.join("ckeys.ron"),
            ckeys: RwLock::new(HashMap::new()),
        }
    }

    pub async fn load(&self) -> Result<(), Error> {
        let path = self.path.clone();
        let ckeys = tokio::task::spawn_blocking(move || {
            let file = match File::open(path) {
                Ok(file) => file,
                Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
                Err(e) => return Err(Error::Read(e.into())),
            };
            let reader = BufReader::new(file);

            Ok(Some(ron::de::from_reader(reader).map_err(Error::Read)?))
        })
        .await
        .expect("Thread panicked")?;

        *self.ckeys.write().await = match ckeys {
            Some(ckeys) => ckeys,
            None => HashMap::new(),
        };

        Ok(())
    }

    pub async fn store(&self) -> Result<(), Error> {
        let path = self.path.clone();
        let value = {
            let guard = self.ckeys.read().await;
            guard.clone()
        };

        tokio::task::spawn_blocking(move || {
            let file = File::create(path).map_err(|e| Error::Write(e.into()))?;
            let mut writer = BufWriter::new(file);

            ron::ser::to_writer_pretty(&mut writer, &value, PrettyConfig::default())
                .map_err(Error::Write)?;

            writer.flush().map_err(|e| Error::Write(e.into()))
        })
        .await
        .expect("Thread panicked")
    }

    pub async fn insert(&self, id: UserId, ckey: &Ckey) -> Result<bool, Error> {
        let result = {
            let mut guard = self.ckeys.write().await;
            guard.insert(id, ckey.as_str().to_string())
        };
        match result {
            None => {
                self.store().await?;

                Ok(true)
            }
            Some(old_ckey) => {
                if old_ckey != ckey.as_str() {
                    self.store().await?;

                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    pub async fn remove(&self, id: &UserId) -> Result<bool, Error> {
        let result = {
            let mut guard = self.ckeys.write().await;
            guard.remove(id)
        };
        if result.is_some() {
            self.store().await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn get_ckey(&self, id: &UserId) -> Option<Ckey> {
        self.ckeys.read().await.get(id).map(Ckey::from)
    }

    pub async fn get_user(&self, ckey: &Ckey) -> Option<UserId> {
        for (id, mapped_ckey) in &*self.ckeys.read().await {
            if mapped_ckey == ckey.as_str() {
                return Some(*id);
            }
        }

        None
    }
}
