use dialoguer::{theme::Theme, Select};
use serenity::{
    http::Http,
    model::prelude::{ChannelType, GuildChannel, GuildInfo},
};

use super::{Error, Wizard};

impl<T: Theme> Wizard<T> {
    pub async fn select_text_channel(
        &self,
        prompt: &str,
        client: &Http,
        guild: &GuildInfo,
    ) -> Result<GuildChannel, Error> {
        loop {
            let channels = client.get_channels(guild.id.0).await?;
            let channels: Vec<_> = channels
                .iter()
                .filter(|c| c.kind == ChannelType::Text)
                .map(|c| (channels.iter().find(|&p| c.parent_id == Some(p.id)), c))
                .collect();
            let options: Vec<_> = channels
                .iter()
                .map(|(p, c)| match *p {
                    Some(p) => format!("[{}] {}", p.name, c.name),
                    None => c.name.clone(),
                })
                .chain(std::iter::once("# Refresh channel list".into()))
                .collect();

            let selection = Select::with_theme(&self.theme)
                .with_prompt(prompt)
                .items(&options)
                .default(0)
                .interact()
                .unwrap();

            if selection >= channels.len() {
                continue;
            } else {
                return Ok(channels[selection].1.clone());
            }
        }
    }
}
