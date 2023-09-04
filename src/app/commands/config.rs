use poise::serenity_prelude as serenity;

use crate::app::{Context, Error};

/// Set config options
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR",
    subcommands(
        "reload",
        "greeting_channel",
        "greeting_message",
        "log_channel",
        "verified_role",
        "verified_message",
        "rejected_message",
        "ckey_prompt"
    )
)]
pub async fn config(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(|b| b.ephemeral(true).content("Use one of the subcommands"))
        .await?;
    Ok(())
}

/// Reload the config from disk
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn reload(ctx: Context<'_>) -> Result<(), Error> {
    ctx.data().config.load().await?;

    ctx.send(|b| b.ephemeral(true).content("Config reloaded from disk"))
        .await?;

    Ok(())
}

/// Set the greeting channel
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn greeting_channel(
    ctx: Context<'_>,
    #[description = "Channel to greet newcomers in"]
    #[channel_types("Text")]
    channel: serenity::GuildChannel,
) -> Result<(), Error> {
    let config = &ctx.data().config;
    config.config.write().await.greeting_channel_id = channel.id;
    config.store().await?;

    ctx.send(|b| {
        b.ephemeral(true)
            .content(format!("Greeting channel set to {channel}"))
    })
    .await?;

    Ok(())
}

/// Set the greeting message
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn greeting_message(
    ctx: Context<'_>,
    #[description = "Message to send with newcomer ping"] string: String,
) -> Result<(), Error> {
    let config = &ctx.data().config;
    config.config.write().await.messages.greeting = string.clone();
    config.store().await?;

    ctx.send(|b| {
        b.ephemeral(true)
            .content(format!("Greeting message set to `{string}`"))
    })
    .await?;

    Ok(())
}

/// Set the greeting message
#[poise::command(
    context_menu_command = "Use as greeting",
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn config_greeting_message_ctx(
    ctx: Context<'_>,
    msg: serenity::Message,
) -> Result<(), Error> {
    let config = &ctx.data().config;
    config.config.write().await.messages.greeting = msg.content;
    config.store().await?;

    ctx.send(|b| b.ephemeral(true).content(format!("Greeting message set")))
        .await?;

    Ok(())
}

/// Set the log channel
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn log_channel(
    ctx: Context<'_>,
    #[description = "Channel to log bot events to"]
    #[channel_types("Text")]
    channel: serenity::GuildChannel,
) -> Result<(), Error> {
    let config = &ctx.data().config;
    config.config.write().await.log_channel_id = channel.id;
    config.store().await?;

    ctx.send(|b| {
        b.ephemeral(true)
            .content(format!("Log channel set to {channel}"))
    })
    .await?;

    Ok(())
}

/// Set the verified role
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn verified_role(
    ctx: Context<'_>,
    #[description = "Role to give to users who pass verification"] role: serenity::Role,
) -> Result<(), Error> {
    let config = &ctx.data().config;
    config.config.write().await.verified_role_id = role.id;
    config.store().await?;

    ctx.send(|b| {
        b.ephemeral(true)
            .content(format!("Verified role set to `{}`", role.name))
    })
    .await?;

    Ok(())
}

/// Set the verified message
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn verified_message(
    ctx: Context<'_>,
    #[description = "Message to send to users who pass verification"] string: String,
) -> Result<(), Error> {
    let config = &ctx.data().config;
    config.config.write().await.messages.verified = string.clone();
    config.store().await?;

    ctx.send(|b| {
        b.ephemeral(true)
            .content(format!("Verified message set to `{string}`"))
    })
    .await?;

    Ok(())
}

/// Set the verified message
#[poise::command(
    context_menu_command = "Use as verified",
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn config_verified_message_ctx(
    ctx: Context<'_>,
    msg: serenity::Message,
) -> Result<(), Error> {
    let config = &ctx.data().config;
    config.config.write().await.messages.verified = msg.content;
    config.store().await?;

    ctx.send(|b| b.ephemeral(true).content(format!("Verified message set")))
        .await?;

    Ok(())
}

/// Set the rejected message
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn rejected_message(
    ctx: Context<'_>,
    #[description = "Message to send to users who fail verification"] string: String,
) -> Result<(), Error> {
    let config = &ctx.data().config;
    config.config.write().await.messages.rejected = string.clone();
    config.store().await?;

    ctx.send(|b| {
        b.ephemeral(true)
            .content(format!("Rejected message set to `{string}`"))
    })
    .await?;

    Ok(())
}

/// Set the rejected message
#[poise::command(
    context_menu_command = "Use as rejected",
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn config_rejected_message_ctx(
    ctx: Context<'_>,
    msg: serenity::Message,
) -> Result<(), Error> {
    let config = &ctx.data().config;
    config.config.write().await.messages.rejected = msg.content;
    config.store().await?;

    ctx.send(|b| b.ephemeral(true).content(format!("Rejected message set")))
        .await?;

    Ok(())
}

/// Set the ckey prompt
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn ckey_prompt(
    ctx: Context<'_>,
    #[description = "Message to prompt users for their ckey with"] string: String,
) -> Result<(), Error> {
    let config = &ctx.data().config;
    config.config.write().await.ckey_prompt = string.clone();
    config.store().await?;

    ctx.send(|b| {
        b.ephemeral(true)
            .content(format!("Ckey prompt set to `{string}`"))
    })
    .await?;

    Ok(())
}
