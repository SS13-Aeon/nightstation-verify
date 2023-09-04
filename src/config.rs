mod wizard;

use std::{
    fmt,
    fs::File,
    io::{self, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use ron::{error::SpannedError, ser::PrettyConfig};
use serde::{Deserialize, Serialize};
use serenity::model::prelude::{ApplicationId, ChannelId, GuildId, RoleId, UserId};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Question {
    pub id: String,
    pub prompt: String,
    pub answers: Vec<String>,
    pub correct_answer: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Messages {
    pub greeting: String,
    pub verified: String,
    pub rejected: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub token: String,
    pub application_id: ApplicationId,
    pub owner_id: UserId,
    pub active_guild_id: GuildId,
    pub greeting_channel_id: ChannelId,
    pub messages: Messages,
    pub log_channel_id: ChannelId,
    pub verified_role_id: RoleId,
    pub ckey_prompt: String,
    pub questions: Vec<Question>,
    pub whitelist_path: PathBuf,
}

pub enum Error {
    Io(io::Error),
    Parse(SpannedError),
    Serialize(ron::Error),
    DiscordError(serenity::Error),
    WizardDismissed,
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<SpannedError> for Error {
    fn from(value: SpannedError) -> Self {
        Self::Parse(value)
    }
}

impl From<ron::Error> for Error {
    fn from(value: ron::Error) -> Self {
        Self::Serialize(value)
    }
}

impl From<serenity::Error> for Error {
    fn from(value: serenity::Error) -> Self {
        Self::DiscordError(value)
    }
}

impl From<wizard::Error> for Error {
    fn from(value: wizard::Error) -> Self {
        match value {
            wizard::Error::Discord(e) => Self::DiscordError(e),
            wizard::Error::Dismissed => Self::WizardDismissed,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::Io(ref e) => fmt::Display::fmt(e, f),
            Error::Parse(ref e) => fmt::Display::fmt(e, f),
            Error::Serialize(ref e) => fmt::Display::fmt(e, f),
            Error::DiscordError(ref e) => fmt::Display::fmt(e, f),
            Error::WizardDismissed => write!(f, "Config wizard dismissed"),
        }
    }
}

impl AppConfig {
    pub async fn load(path: &Path) -> Result<AppConfig, Error> {
        match File::open(path) {
            Ok(f) => {
                let reader = BufReader::new(f);

                Ok(ron::de::from_reader(reader)?)
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                log::info!("No config file found");

                let config = wizard::run().await?;
                config.store(path)?;

                return Ok(config);
            }
            Err(e) => return Err(e.into()),
        }
    }

    pub fn store(&self, path: &Path) -> Result<(), ron::Error> {
        let file = File::create(path)?;

        let mut writer = BufWriter::new(file);

        ron::ser::to_writer_pretty(&mut writer, self, PrettyConfig::default())?;

        Ok(writer.flush()?)
    }
}
