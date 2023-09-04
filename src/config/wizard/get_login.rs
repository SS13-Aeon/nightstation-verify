use dialoguer::{console::style, theme::Theme, Password};
use serenity::{http::Http, model::prelude::CurrentUser};

use super::Wizard;

impl<T: Theme> Wizard<T> {
    pub async fn get_login(&self) -> (Http, CurrentUser) {
        eprintln!(
            "Visit {} and create a new application",
            style("https://discord.com/developers/applications").underlined(),
        );
        eprintln!("The token prompt will not react as you're typing");

        loop {
            let token: String = Password::with_theme(&self.theme)
                .with_prompt("Enter your bot token")
                .interact()
                .unwrap();

            let client = Http::new(&token);

            match client.get_current_user().await {
                Ok(user) => break (client, user),
                Err(e) => {
                    eprintln!("{} {}", style("Failed to log in:").red().bold(), e);

                    continue;
                }
            };
        }
    }
}
