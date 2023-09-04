mod check_application;
mod get_login;
mod get_questions;
mod get_whitelist_path;
mod select_active_guild;
mod select_role;
mod select_text_channel;

use dialoguer::{
    console::style,
    theme::{ColorfulTheme, Theme},
    Confirm, Input,
};

use crate::{config::Messages, AppConfig};

pub enum Error {
    Discord(serenity::Error),
    Dismissed,
}

impl From<serenity::Error> for Error {
    fn from(value: serenity::Error) -> Self {
        Self::Discord(value)
    }
}

struct Wizard<T: Theme> {
    theme: T,
}

impl<T: Theme> Wizard<T> {
    fn new(theme: T) -> Wizard<T> {
        Wizard { theme }
    }

    fn confirm_start(&self) -> bool {
        Confirm::with_theme(&self.theme)
            .default(true)
            .with_prompt("Would you like to initialize a new config?")
            .interact()
            .unwrap()
    }

    fn get_text(&self, prompt: &str) -> String {
        Input::with_theme(&self.theme)
            .with_prompt(prompt)
            .interact_text()
            .unwrap()
    }
}

pub async fn run() -> Result<AppConfig, Error> {
    let wizard = Wizard::new(ColorfulTheme::default());

    if !dialoguer::console::user_attended_stderr() || !wizard.confirm_start() {
        return Err(Error::Dismissed);
    }

    eprintln!("{}", style("[1/9] Login info").cyan().bold());

    let (client, user) = wizard.get_login().await;

    eprintln!(
        "{} {}",
        style("Successfully logged in as").green(),
        user.name
    );

    eprintln!(
        "{}",
        style("[2/9] Application settings sanity check")
            .cyan()
            .bold()
    );

    let info = wizard.check_application(&client).await?;

    eprintln!(
        "{} {} {} {} {}",
        style("Application").green(),
        info.name,
        style("owned by").green(),
        info.owner.name,
        style("is correctly configured").green()
    );

    eprintln!("{}", style("[3/9] Active server").cyan().bold());

    let active_guild = wizard.select_active_guild(&client, &info).await?;

    eprintln!("{}", style("[4/9] Greeting channel").cyan().bold());

    let greeting_channel = wizard
        .select_text_channel(
            "Select which channel this bot should ping newcomers in",
            &client,
            &active_guild,
        )
        .await?;

    eprintln!("{}", style("[5/9] Messages").cyan().bold());

    eprintln!("The bot will send this message with every newcomer ping");
    let greeting_message = wizard.get_text("Enter greeting message");

    eprintln!("The bot will send this message once the newcomer is verified");
    let verified_message = wizard.get_text("Enter verified message");

    eprintln!("The bot will send this message if the newcomer fails verification");
    let rejected_message = wizard.get_text("Enter rejected message");

    eprintln!("{}", style("[6/9] Log channel").cyan().bold());

    let log_channel = wizard
        .select_text_channel(
            "Select which channel this bot should log to",
            &client,
            &active_guild,
        )
        .await?;

    eprintln!("{}", style("[7/9] Verified role").cyan().bold());

    let verified_role = wizard
        .select_role(
            "Select which role should be applied to verified users",
            &client,
            &active_guild,
        )
        .await?;

    eprintln!("{}", style("[8/9] Questions and answers").cyan().bold());

    let (ckey_prompt, questions) = wizard.get_questions();

    eprintln!("{}", style("[9/9] Whitelist location").cyan().bold());

    let whitelist_path = wizard.get_whitelist_path();

    Ok(AppConfig {
        token: client.token,
        application_id: info.id,
        owner_id: info.owner.id,
        active_guild_id: active_guild.id,
        greeting_channel_id: greeting_channel.id,
        messages: Messages {
            greeting: greeting_message,
            verified: verified_message,
            rejected: rejected_message,
        },
        log_channel_id: log_channel.id,
        verified_role_id: verified_role.id,
        ckey_prompt,
        questions,
        whitelist_path,
    })
}
