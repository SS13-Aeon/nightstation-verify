use poise::serenity_prelude as serenity;
use poise::{BoxFuture, FrameworkContext};

use crate::app::{Data, Error};

pub fn event_handler<'a>(
    sc: &'a serenity::Context,
    event: &'a poise::Event<'a>,
    ctx: FrameworkContext<'a, Data, Error>,
    _data: &'a Data,
) -> BoxFuture<'a, Result<(), Error>> {
    Box::pin(async move {
        match *event {
            poise::Event::GuildMemberAddition { ref new_member } => {
                ctx.user_data
                    .verification
                    .on_join(sc, ctx, new_member)
                    .await
            }
            poise::Event::GuildBanAddition {
                guild_id,
                ref banned_user,
            } => {
                ctx.user_data
                    .verification
                    .on_ban(sc, ctx, guild_id, banned_user)
                    .await
            }
            poise::Event::InteractionCreate { ref interaction } => {
                ctx.user_data
                    .verification
                    .on_interaction(sc, ctx, interaction)
                    .await
            }
            _ => Ok(()),
        }
    })
}
