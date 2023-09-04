use dialoguer::{console::style, theme::Theme, Confirm};
use serenity::{
    http::Http,
    model::prelude::{ApplicationFlags, CurrentApplicationInfo},
};

use super::{Error, Wizard};

impl<T: Theme> Wizard<T> {
    pub async fn check_application(&self, client: &Http) -> Result<CurrentApplicationInfo, Error> {
        loop {
            let info = client.get_current_application_info().await?;

            let mut found_errors = false;
            if info.bot_public {
                found_errors = true;
                eprintln!(
                    "{}",
                    style("Your bot is set to public, allowing anyone to add it to their server")
                        .red()
                        .bold()
                );
            } else {
                eprintln!("{}", style("Bot is set to private").green());
            }

            if let Some(flags) = info.flags {
                if !flags.intersects(
                    ApplicationFlags::GATEWAY_GUILD_MEMBERS
                        | ApplicationFlags::GATEWAY_GUILD_MEMBERS_LIMITED,
                ) {
                    found_errors = true;
                    eprintln!(
                        "{}",
                        style("Your bot doesn't have the server members intent enabled")
                            .red()
                            .bold()
                    );
                } else {
                    eprintln!("{}", style("Bot has server members intent enabled").green());
                }
                if !flags.intersects(
                    ApplicationFlags::GATEWAY_MESSAGE_CONTENT
                        | ApplicationFlags::GATEWAY_MESSAGE_CONTENT_LIMITED,
                ) {
                    found_errors = true;
                    eprintln!(
                        "{}",
                        style("Your bot doesn't have the message content intent enabled")
                            .red()
                            .bold()
                    );
                } else {
                    eprintln!(
                        "{}",
                        style("Bot has message content intent enabled").green()
                    );
                }
            }

            if found_errors {
                if Confirm::with_theme(&self.theme)
                    .default(true)
                    .with_prompt(
                        "Your application is misconfigured, would you like to check again?",
                    )
                    .interact()
                    .unwrap()
                {
                    continue;
                } else {
                    break Err(Error::Dismissed);
                }
            }

            break Ok(info);
        }
    }
}
