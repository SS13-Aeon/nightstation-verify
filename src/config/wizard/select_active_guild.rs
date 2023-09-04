use dialoguer::{console::style, theme::Theme, Select};
use serenity::{
    http::Http,
    model::{
        prelude::{CurrentApplicationInfo, GuildInfo},
        Permissions,
    },
};
use url::Url;

use super::{Error, Wizard};

impl<T: Theme> Wizard<T> {
    pub async fn select_active_guild(
        &self,
        client: &Http,
        app: &CurrentApplicationInfo,
    ) -> Result<GuildInfo, Error> {
        let permissions = Permissions::SEND_MESSAGES
            | Permissions::MANAGE_MESSAGES
            | Permissions::READ_MESSAGE_HISTORY
            | Permissions::MANAGE_ROLES;
        let invite_link: Url = Url::parse_with_params(
            "https://discord.com/api/oauth2/authorize",
            &[
                ("client_id", app.id.to_string()),
                ("permissions", permissions.bits().to_string()),
                ("scope", "bot applications.commands".into()),
            ],
        )
        .expect("Failed to generate invite link");

        eprintln!(
            "Visit {} and add your bot to your server",
            style(invite_link).underlined()
        );

        loop {
            let guilds = client.get_guilds(None, None).await?;
            let options: Vec<_> = guilds
                .iter()
                .map(|g| &g.name[..])
                .chain(std::iter::once("# Refresh server list"))
                .collect();

            let selection = Select::with_theme(&self.theme)
                .with_prompt("Select which server this bot should manage")
                .items(&options)
                .default(0)
                .interact()
                .unwrap();

            if selection >= guilds.len() {
                continue;
            } else {
                return Ok(guilds[selection].clone());
            }
        }
    }
}
