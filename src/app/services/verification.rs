use std::{
    collections::HashMap,
    fmt,
    fs::File,
    io::{self, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    sync::{Arc, Weak},
};

use poise::serenity_prelude as serenity;
use poise::FrameworkContext;
use ron::{error::SpannedError, ser::PrettyConfig};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::app::{models::Ckey, Data, Error as AppError};

use super::{CkeyService, ConfigService, WhitelistService};

#[derive(Debug)]
pub enum Error {
    Read(SpannedError),
    Write(ron::Error),
    Dependency(&'static str),
    GrantRole(serenity::Error),
    SendGreeting(SendGreetingError),
}

impl From<Error> for super::Error {
    fn from(value: Error) -> Self {
        Self::Verification(value)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::Read(ref e) => write!(f, "read: {e}"),
            Error::Write(ref e) => write!(f, "write: {e}"),
            Error::Dependency(ref e) => write!(f, "dependency not loaded: {e}"),
            Error::GrantRole(ref e) => write!(f, "grant_role: {e}"),
            Error::SendGreeting(ref e) => write!(f, "send_greeting: {e}"),
        }
    }
}

#[derive(Debug)]
pub enum SendGreetingError {
    AlreadyVerified,
    AlreadyPending,
    AlreadyRejected,
    Discord(serenity::Error),
}

impl From<SendGreetingError> for Error {
    fn from(value: SendGreetingError) -> Self {
        Self::SendGreeting(value)
    }
}

impl fmt::Display for SendGreetingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::AlreadyVerified => write!(f, "Already verified"),
            Self::AlreadyPending => write!(f, "Verification already pending"),
            Self::AlreadyRejected => write!(f, "Already rejected"),
            Self::Discord(ref e) => fmt::Display::fmt(e, f),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum VerificationStatus {
    Greeted {
        greeting_id: serenity::MessageId,
    },
    Pending {
        form_data: Vec<String>,
        form_idx: usize,
    },
    Verified,
    Rejected,
}

pub struct VerificationService {
    path: PathBuf,
    ckey: Weak<CkeyService>,
    config: Weak<ConfigService>,
    whitelist: Weak<WhitelistService>,
    data: RwLock<HashMap<serenity::UserId, VerificationStatus>>,
}

impl VerificationService {
    pub fn new(
        ckey: Weak<CkeyService>,
        config: Weak<ConfigService>,
        whitelist: Weak<WhitelistService>,
        data_path: &Path,
    ) -> Self {
        Self {
            path: data_path.join("verification.ron"),
            ckey,
            config,
            whitelist,
            data: RwLock::new(HashMap::new()),
        }
    }

    pub async fn load(&self) -> Result<(), Error> {
        let path = self.path.clone();
        let data = tokio::task::spawn_blocking(move || {
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

        *self.data.write().await = match data {
            Some(data) => data,
            None => HashMap::new(),
        };

        Ok(())
    }

    pub async fn store(&self) -> Result<(), Error> {
        let path = self.path.clone();
        let value = {
            let guard = self.data.read().await;
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
        .expect("Thead panicked")
    }

    pub async fn set_status(
        &self,
        id: serenity::UserId,
        status: &VerificationStatus,
    ) -> Result<bool, Error> {
        let result = {
            let mut guard = self.data.write().await;
            guard.insert(id, status.clone())
        };

        match result {
            None => {
                self.store().await?;

                Ok(true)
            }
            Some(old_status) => {
                if old_status != *status {
                    self.store().await?;

                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    pub async fn get_status(&self, id: &serenity::UserId) -> Option<VerificationStatus> {
        let guard = self.data.read().await;
        guard.get(id).cloned()
    }

    pub async fn remove(&self, id: &serenity::UserId) -> Result<bool, Error> {
        let result = {
            let mut guard = self.data.write().await;
            guard.remove(id)
        };
        if result.is_some() {
            self.store().await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn config(&self) -> Result<Arc<ConfigService>, Error> {
        self.config.upgrade().ok_or(Error::Dependency("config"))
    }

    fn ckey(&self) -> Result<Arc<CkeyService>, Error> {
        self.ckey.upgrade().ok_or(Error::Dependency("ckey"))
    }

    fn whitelist(&self) -> Result<Arc<WhitelistService>, Error> {
        self.whitelist
            .upgrade()
            .ok_or(Error::Dependency("whitelist"))
    }

    pub async fn grant_role(
        &self,
        sc: &serenity::Context,
        member: &mut serenity::Member,
    ) -> Result<(), Error> {
        let verified_role_id = {
            let config = self.config()?;
            let guard = config.get().await;
            guard.verified_role_id
        };

        member
            .add_role(sc, verified_role_id)
            .await
            .map_err(Error::GrantRole)?;

        Ok(())
    }

    pub async fn send_greeting(
        &self,
        sc: &serenity::Context,
        user: &serenity::User,
    ) -> Result<serenity::MessageId, Error> {
        let (greeting_channel_id, greeting_message) = {
            let config = self.config()?;
            let guard = config.get().await;
            (guard.greeting_channel_id, guard.messages.greeting.clone())
        };
        let status = self.get_status(&user.id).await;

        match status {
            Some(VerificationStatus::Greeted { greeting_id }) => Ok(greeting_id),
            Some(VerificationStatus::Pending {
                form_data: _,
                form_idx: _,
            }) => Err(SendGreetingError::AlreadyPending.into()),
            Some(VerificationStatus::Verified) => Err(SendGreetingError::AlreadyVerified.into()),
            Some(VerificationStatus::Rejected) => Err(SendGreetingError::AlreadyRejected.into()),
            None => {
                let message = greeting_channel_id
                    .send_message(sc, |b| {
                        b.allowed_mentions(|b| b.users([user.id]))
                            .content(format!("{user}: {greeting_message}"))
                            .components(|b| {
                                b.create_action_row(|b| {
                                    b.create_button(|b| {
                                        b.custom_id("begin_verification")
                                            .label("Begin")
                                            .style(serenity::ButtonStyle::Primary)
                                    })
                                })
                            })
                    })
                    .await
                    .map_err(SendGreetingError::Discord)?;

                self.set_status(
                    user.id,
                    &VerificationStatus::Greeted {
                        greeting_id: message.id,
                    },
                )
                .await?;

                Ok(message.id)
            }
        }
    }

    pub async fn render_form(
        &self,
        sc: &serenity::Context,
        interaction: &serenity::MessageComponentInteraction,
        form_idx: usize,
    ) -> Result<(), AppError> {
        let questions = {
            let config = self.config()?;
            let guard = config.get().await;
            guard.questions.clone()
        };

        let current_question = &questions[form_idx];

        let is_ephemeral = interaction
            .message
            .flags
            .unwrap_or(serenity::MessageFlags::empty())
            .contains(serenity::MessageFlags::EPHEMERAL);

        interaction
            .create_interaction_response(sc, |b| {
                b.kind(if is_ephemeral {
                    serenity::InteractionResponseType::UpdateMessage
                } else {
                    serenity::InteractionResponseType::ChannelMessageWithSource
                })
                .interaction_response_data(|b| {
                    b.ephemeral(true)
                        .content(&current_question.prompt)
                        .components(|b| {
                            b.create_action_row(|b| {
                                b.create_select_menu(|b| {
                                    b.custom_id("form_answer")
                                        .placeholder("Answer")
                                        .options(|b| {
                                            for answer in &current_question.answers {
                                                b.create_option(|b| b.label(answer).value(answer));
                                            }

                                            b
                                        })
                                })
                            })
                        })
                })
            })
            .await?;

        if !is_ephemeral {
            interaction.message.delete(sc).await?;
        }

        Ok(())
    }

    pub async fn reject(&self, user_id: &serenity::UserId) -> Result<(), AppError> {
        self.set_status(*user_id, &VerificationStatus::Rejected)
            .await?;

        Ok(())
    }

    pub async fn verify(&self, user_id: &serenity::UserId, ckey: Ckey) -> Result<(), AppError> {
        self.set_status(*user_id, &VerificationStatus::Verified)
            .await?;
        {
            let ckey_service = self.ckey()?;
            ckey_service.insert(*user_id, &ckey).await?;
        }

        {
            let whitelist_service = self.whitelist()?;
            whitelist_service.insert(&ckey).await?;
        }

        Ok(())
    }

    pub async fn validate_form(
        &self,
        sc: &serenity::Context,
        interaction: &serenity::MessageComponentInteraction,
        form_data: &Vec<String>,
    ) -> Result<(), AppError> {
        let (questions, rejected_message, ckey_prompt, log_channel_id) = {
            let config = self.config()?;
            let guard = config.get().await;

            (
                guard.questions.clone(),
                guard.messages.rejected.clone(),
                guard.ckey_prompt.clone(),
                guard.log_channel_id,
            )
        };

        let valid = form_data.len() == questions.len()
            && questions
                .iter()
                .enumerate()
                .all(|(i, q)| form_data[i] == q.answers[q.correct_answer]);

        if !valid {
            self.reject(&interaction.user.id).await?;

            interaction
                .create_interaction_response(sc, |b| {
                    b.kind(serenity::InteractionResponseType::UpdateMessage)
                        .interaction_response_data(|b| {
                            b.ephemeral(true)
                                .content(rejected_message)
                                .set_components(serenity::CreateComponents::default())
                        })
                })
                .await?;

            log_channel_id
                .send_message(sc, |b| {
                    b.content(format!(
                        "{} sent invalid form answers:\n```json\n{:#?}\n```",
                        &interaction.user, form_data
                    ))
                })
                .await?;

            return Ok(());
        }

        interaction
            .create_interaction_response(sc, |b| {
                b.kind(serenity::InteractionResponseType::Modal)
                    .interaction_response_data(|b| {
                        b.custom_id("ckey_modal")
                            .title("Complete verification")
                            .components(|b| {
                                b.create_action_row(|b| {
                                    b.create_input_text(|b| {
                                        b.custom_id("ckey")
                                            .label(ckey_prompt)
                                            .required(true)
                                            .style(serenity::InputTextStyle::Short)
                                    })
                                })
                            })
                    })
            })
            .await?;

        log_channel_id
            .send_message(sc, |b| {
                b.content(format!(
                    "{} sent valid form answers:\n```json\n{:#?}\n```",
                    &interaction.user, form_data
                ))
            })
            .await?;

        Ok(())
    }

    pub async fn on_join<'a>(
        &self,
        sc: &'a serenity::Context,
        ctx: FrameworkContext<'a, Data, AppError>,
        new_member: &serenity::Member,
    ) -> Result<(), AppError> {
        let (active_guild_id, log_channel_id) = {
            let config = ctx.user_data.config.get().await;
            (config.active_guild_id, config.log_channel_id)
        };

        if active_guild_id != new_member.guild_id {
            return Ok(());
        }

        let greeting_result = self.send_greeting(sc, &new_member.user).await;

        match greeting_result {
            Ok(_) => Ok(()),
            Err(Error::SendGreeting(SendGreetingError::AlreadyVerified)) => {
                let ckey = {
                    let ckey = self.ckey()?;
                    ckey.get_ckey(&new_member.user.id).await
                };

                match ckey {
                    Some(ckey) => {
                        self.grant_role(sc, &mut new_member.clone()).await?;

                        log_channel_id.send_message(sc, |b| {
                            b.content(format!("{new_member} (ID: {}, ckey: {ckey}) joined with existing verification, role granted", new_member.user.id))
                        }).await?
                    }
                    None => {
                        self.remove(&new_member.user.id).await?;
                        self.send_greeting(sc, &new_member.user).await?;

                        log_channel_id.send_message(sc, |b| {
                            b.content(format!("{new_member} (ID: {}) joined with existing verification, but no ckey", new_member.user.id))
                        }).await?
                    }
                };

                Ok(())
            }
            Err(Error::SendGreeting(SendGreetingError::AlreadyPending)) => {
                self.remove(&new_member.user.id).await?;
                self.send_greeting(sc, &new_member.user).await?;

                log_channel_id
                    .send_message(sc, |b| {
                        b.content(format!(
                            "{new_member} (ID: {}) joined with unfinished verification",
                            new_member.user.id
                        ))
                    })
                    .await?;

                Ok(())
            }
            Err(Error::SendGreeting(SendGreetingError::AlreadyRejected)) => {
                log_channel_id
                    .send_message(sc, |b| {
                        b.content(format!(
                            "{new_member} (ID: {}) joined with rejected verification",
                            new_member.user.id
                        ))
                    })
                    .await?;

                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    pub async fn on_ban<'a>(
        &self,
        sc: &'a serenity::Context,
        ctx: FrameworkContext<'a, Data, AppError>,
        guild_id: serenity::GuildId,
        banned_user: &'a serenity::User,
    ) -> Result<(), AppError> {
        let (active_guild_id, log_channel_id) = {
            let config = ctx.user_data.config.get().await;
            (config.active_guild_id, config.log_channel_id)
        };

        if active_guild_id != guild_id {
            return Ok(());
        }

        self.remove(&banned_user.id).await?;

        let ckey = ctx.user_data.ckey.get_ckey(&banned_user.id).await;

        match ckey {
            Some(ckey) => {
                let result = ctx.user_data.whitelist.remove(&ckey).await?;

                log_channel_id
                    .send_message(sc, |b| {
                        b.content(format!(
                            "`{}` (ID: `{}`, ckey: `{ckey}`) banned{}",
                            banned_user.name,
                            banned_user.id,
                            if result {
                                " and removed from whitelist"
                            } else {
                                ""
                            }
                        ))
                    })
                    .await?;
            }
            None => {
                log_channel_id
                    .send_message(sc, |b| {
                        b.content(format!(
                            "`{}` (ID: `{}`, no ckey) banned",
                            banned_user.name, banned_user.id
                        ))
                    })
                    .await?;
            }
        };

        Ok(())
    }

    pub async fn on_interaction<'a>(
        &self,
        sc: &'a serenity::Context,
        ctx: FrameworkContext<'a, Data, AppError>,
        interaction: &'a serenity::Interaction,
    ) -> Result<(), AppError> {
        let (active_guild_id, log_channel_id, questions_len, verified_message) = {
            let config = ctx.user_data.config.get().await;
            (
                config.active_guild_id,
                config.log_channel_id,
                config.questions.len(),
                config.messages.verified.clone(),
            )
        };

        match *interaction {
            serenity::Interaction::MessageComponent(ref interaction) => {
                if interaction.guild_id != Some(active_guild_id) {
                    return Ok(());
                }

                match interaction.data.custom_id.as_str() {
                    "begin_verification" => {
                        let user = &interaction.user;

                        let Some(VerificationStatus::Greeted { greeting_id }) =
                            self.get_status(&user.id).await
                        else {
                            return Ok(());
                        };

                        if interaction.message.id != greeting_id {
                            return Ok(());
                        }

                        if questions_len == 0 {
                            self.validate_form(sc, interaction, &Vec::new()).await?;

                            return Ok(());
                        }

                        self.render_form(sc, interaction, 0).await?;

                        let new_status = VerificationStatus::Pending {
                            form_data: Vec::new(),
                            form_idx: 0,
                        };

                        self.set_status(user.id, &new_status).await?;

                        log_channel_id
                            .send_message(sc, |b| b.content(format!("{user} started verification")))
                            .await?;

                        Ok(())
                    }
                    "form_answer" => {
                        let user = &interaction.user;

                        let Some(VerificationStatus::Pending {
                            mut form_data,
                            form_idx,
                        }) = self.get_status(&user.id).await
                        else {
                            return Ok(());
                        };

                        if interaction.data.values.len() != 1 {
                            return Ok(());
                        }

                        form_data.push(interaction.data.values[0].clone());

                        let form_idx = form_idx + 1;

                        if form_idx >= questions_len {
                            self.validate_form(sc, interaction, &form_data).await?;

                            return Ok(());
                        }

                        self.render_form(sc, interaction, form_idx).await?;

                        let new_status = VerificationStatus::Pending {
                            form_data,
                            form_idx,
                        };

                        self.set_status(user.id, &new_status).await?;

                        Ok(())
                    }
                    _ => Ok(()),
                }
            }
            serenity::Interaction::ModalSubmit(ref interaction) => {
                if interaction.guild_id != Some(active_guild_id) {
                    return Ok(());
                }

                match interaction.data.custom_id.as_str() {
                    "ckey_modal" => {
                        let Some(ref member) = interaction.member else {
                            return Ok(());
                        };

                        let status = self.get_status(&member.user.id).await;

                        match status {
                            Some(
                                VerificationStatus::Greeted { greeting_id: _ }
                                | VerificationStatus::Pending {
                                    form_data: _,
                                    form_idx: _,
                                },
                            ) => (),
                            _ => return Ok(()),
                        };

                        if interaction.data.components.len() != 1
                            && interaction.data.components[0].components.len() != 1
                        {
                            return Ok(());
                        }

                        let serenity::ActionRowComponent::InputText(serenity::InputText {
                            kind: _,
                            custom_id,
                            value,
                        }) = &interaction.data.components[0].components[0]
                        else {
                            return Ok(());
                        };

                        if custom_id != "ckey" {
                            return Ok(());
                        }

                        let ckey = Ckey::from(value);

                        self.verify(&member.user.id, ckey.clone()).await?;
                        self.grant_role(sc, &mut member.clone()).await?;

                        interaction
                            .create_interaction_response(sc, |b| {
                                b.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|b| {
                                        b.ephemeral(true).content(verified_message)
                                    })
                            })
                            .await?;

                        log_channel_id
                            .send_message(sc, |b| {
                                b.content(format!("{member} is now verified for ckey `{ckey}`"))
                            })
                            .await?;

                        Ok(())
                    }
                    _ => Ok(()),
                }
            }
            _ => Ok(()),
        }
    }
}
