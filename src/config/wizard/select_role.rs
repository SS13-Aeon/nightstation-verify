use dialoguer::{theme::Theme, Select};
use serenity::{
    http::Http,
    model::prelude::{GuildInfo, Role},
};

use super::{Error, Wizard};

impl<T: Theme> Wizard<T> {
    pub async fn select_role(
        &self,
        prompt: &str,
        client: &Http,
        guild: &GuildInfo,
    ) -> Result<Role, Error> {
        loop {
            let roles = client.get_guild_roles(guild.id.0).await?;
            let options: Vec<_> = roles
                .iter()
                .map(|r| &r.name[..])
                .chain(std::iter::once("# Refresh role list"))
                .collect();

            let selection = Select::with_theme(&self.theme)
                .with_prompt(prompt)
                .items(&options)
                .default(0)
                .interact()
                .unwrap();

            if selection >= roles.len() {
                continue;
            } else {
                return Ok(roles[selection].clone());
            }
        }
    }
}
