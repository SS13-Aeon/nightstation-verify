mod commands;
mod event_handler;
mod models;
mod services;

use std::{fmt, fs, io, path::Path, sync::Arc};

use crate::AppConfig;

use poise::FrameworkError;
use serenity::prelude::GatewayIntents;
use services::{CkeyService, ConfigService, VerificationService, WhitelistService};

pub struct Data {
    ckey: Arc<CkeyService>,
    config: Arc<ConfigService>,
    verification: Arc<VerificationService>,
    whitelist: Arc<WhitelistService>,
}

#[derive(Debug)]
pub enum Error {
    DataDir(io::Error),
    Whietlist(io::Error),
    Discord(serenity::Error),
    Service(services::Error),
    JoinError(tokio::task::JoinError),
}

impl From<serenity::Error> for Error {
    fn from(value: serenity::Error) -> Self {
        Error::Discord(value)
    }
}

impl<T: Into<services::Error>> From<T> for Error {
    fn from(value: T) -> Self {
        Error::Service(value.into())
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(value: tokio::task::JoinError) -> Self {
        Error::JoinError(value)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::DataDir(ref e) => fmt::Display::fmt(e, f),
            Error::Whietlist(ref e) => fmt::Display::fmt(e, f),
            Error::Discord(ref e) => fmt::Display::fmt(e, f),
            Error::Service(ref e) => fmt::Display::fmt(e, f),
            Error::JoinError(ref e) => fmt::Display::fmt(e, f),
        }
    }
}

type Context<'a> = poise::Context<'a, Data, Error>;

pub struct AppPaths<'a> {
    pub config: &'a Path,
    pub data: &'a Path,
}

pub struct App<'a> {
    config: AppConfig,
    paths: AppPaths<'a>,
}

impl<'a> App<'a> {
    pub fn new(config: AppConfig, paths: AppPaths<'a>) -> Self {
        Self { config, paths }
    }

    pub async fn init_services(&self) -> Result<Data, Error> {
        log::info!("Initializing services");

        match fs::create_dir(self.paths.data) {
            Ok(_) => {
                log::info!(
                    "Storing service data in {}",
                    fs::canonicalize(self.paths.data)
                        .map_err(|e| Error::DataDir(e))?
                        .display()
                );
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                log::info!(
                    "Storing service data in {}",
                    fs::canonicalize(self.paths.data)
                        .map_err(|e| Error::DataDir(e))?
                        .display()
                );
            }
            Err(e) => return Err(Error::DataDir(e)),
        }

        let config = Arc::new(ConfigService::new(self.config.clone(), self.paths.config));

        let whitelist = Arc::new(WhitelistService::new(&self.config.whitelist_path));
        whitelist.load().await?;
        let whitelist_path =
            fs::canonicalize(&self.config.whitelist_path).map_err(|e| Error::DataDir(e))?;
        log::info!("Whitelist loaded from {}", whitelist_path.display());

        let ckey = Arc::new(CkeyService::new(self.paths.data));
        ckey.load().await?;
        log::info!("Ckey mapping loaded");

        let verification = Arc::new(VerificationService::new(
            Arc::downgrade(&ckey),
            Arc::downgrade(&config),
            Arc::downgrade(&whitelist),
            self.paths.data,
        ));
        verification.load().await?;
        log::info!("Verifier loaded");

        Ok(Data {
            ckey,
            config,
            verification,
            whitelist,
        })
    }

    pub async fn run(self) -> Result<(), Error> {
        let services = self.init_services().await?;

        log::info!("Initializing Discord client");

        let intents = GatewayIntents::GUILD_MEMBERS
            | GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;

        let framework = poise::Framework::builder()
            .options(poise::FrameworkOptions {
                commands: commands::commands(),
                on_error: |err| {
                    Box::pin(async move {
                        match err {
                            FrameworkError::Setup {
                                error,
                                framework: _,
                                data_about_bot: _,
                                ctx: _,
                            } => log::error!("setup: {error}"),
                            FrameworkError::EventHandler {
                                error,
                                ctx: _,
                                event: _,
                                framework: _,
                            } => log::error!("command: {error}"),
                            FrameworkError::Command { error, ctx: _ } => {
                                log::error!("command: {error}")
                            }
                            FrameworkError::CommandCheckFailed {
                                error: Some(error),
                                ctx: _,
                            } => {
                                log::error!("check failed: {error}")
                            }
                            FrameworkError::DynamicPrefix {
                                error,
                                ctx: _,
                                msg: _,
                            } => log::error!("dynamic prefix: {error}"),
                            _ => (),
                        }
                    })
                },
                event_handler: event_handler::event_handler,
                ..Default::default()
            })
            .token(&self.config.token)
            .intents(intents)
            .setup(move |ctx, _ready, framework| {
                Box::pin(async move {
                    log::info!("Registering commands");

                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                    log::info!("Ready!");

                    Ok(services)
                })
            })
            .build()
            .await?;

        log::info!("Starting bot");

        framework.start().await?;

        Ok(())
    }
}
