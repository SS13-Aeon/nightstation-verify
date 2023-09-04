mod ckey;
mod config;
mod verification;
mod whitelist;

use std::fmt;

pub use ckey::{CkeyService, Error as CkeyError};
pub use config::{ConfigService, Error as ConfigError};
pub use verification::{
    Error as VerificationError, SendGreetingError, VerificationService, VerificationStatus,
};
pub use whitelist::{Error as WhitelistError, WhitelistService};

#[derive(Debug)]
pub enum Error {
    Ckey(ckey::Error),
    Config(config::Error),
    Verification(verification::Error),
    Whitelist(whitelist::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::Ckey(ref e) => write!(f, "ckey: {e}"),
            Error::Config(ref e) => write!(f, "config: {e}"),
            Error::Verification(ref e) => write!(f, "verification: {e}"),
            Error::Whitelist(ref e) => write!(f, "whitelist: {e}"),
        }
    }
}
