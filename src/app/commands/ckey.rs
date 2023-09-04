use poise::serenity_prelude as serenity;

use crate::app::{models::Ckey, Context, Error};

/// Modify and inspect ckey mappings
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR",
    subcommands("reload", "get", "find", "set", "unset")
)]
pub async fn ckey(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(|b| b.ephemeral(true).content("Use one of the subcommands"))
        .await?;
    Ok(())
}

/// Reload the ckey mapping from disk
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn reload(ctx: Context<'_>) -> Result<(), Error> {
    ctx.data().ckey.load().await?;

    ctx.send(|b| b.ephemeral(true).content("Ckey mapping reloaded from disk"))
        .await?;

    Ok(())
}

/// Get user's mapped ckey
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn get(
    ctx: Context<'_>,
    #[description = "Target user"] member: serenity::Member,
) -> Result<(), Error> {
    let result = ctx.data().ckey.get_ckey(&member.user.id).await;

    ctx.send(|b| {
        b.ephemeral(true).content(match result {
            Some(ckey) => format!("{member} is mapped to `{ckey}`"),
            None => format!("{member} has no ckey mapping"),
        })
    })
    .await?;

    Ok(())
}

/// Get user's mapped ckey
#[poise::command(
    context_menu_command = "Get ckey",
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn ckey_get_ctx(ctx: Context<'_>, user: serenity::User) -> Result<(), Error> {
    let result = ctx.data().ckey.get_ckey(&user.id).await;

    ctx.send(|b| {
        b.ephemeral(true).content(match result {
            Some(ckey) => format!("{user} is mapped to `{ckey}`"),
            None => format!("{user} has no ckey mapping"),
        })
    })
    .await?;

    Ok(())
}

/// Find Discord user for ckey
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn find(
    ctx: Context<'_>,
    #[description = "Corresponding ckey"] ckey: String,
) -> Result<(), Error> {
    let ckey = Ckey::from(&ckey);
    let result = ctx.data().ckey.get_user(&ckey).await;
    let response = match result {
        Some(user_id) => match user_id.to_user(ctx).await {
            Ok(user) => format!("`{ckey}` belongs to {user}"),
            Err(_) => format!("`{ckey}` belongs to user with ID `{user_id}`"),
        },
        None => format!("`{ckey}` is not mapped to any user"),
    };

    ctx.send(|b| b.ephemeral(true).content(response)).await?;

    Ok(())
}

/// Set user's ckey mapping
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn set(
    ctx: Context<'_>,
    #[description = "Target user"] member: serenity::Member,
    #[description = "Corresponding ckey"] ckey: String,
) -> Result<(), Error> {
    let ckey = Ckey::from(&ckey);
    let result = ctx.data().ckey.insert(member.user.id, &ckey).await?;

    ctx.send(|b| {
        b.ephemeral(true).content(if result {
            format!("{member} mapped to `{ckey}`")
        } else {
            format!("{member} is already mapped to `{ckey}`")
        })
    })
    .await?;

    Ok(())
}

/// Unset user's ckey mapping
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn unset(
    ctx: Context<'_>,
    #[description = "Target user"] member: serenity::Member,
) -> Result<(), Error> {
    let result = ctx.data().ckey.remove(&member.user.id).await?;

    ctx.send(|b| {
        b.ephemeral(true).content(if result {
            format!("{member} ckey mapping removed")
        } else {
            format!("{member} has no ckey mapping")
        })
    })
    .await?;

    Ok(())
}

/// Unset user's ckey mapping
#[poise::command(
    context_menu_command = "Unset ckey",
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn ckey_unset_ctx(ctx: Context<'_>, user: serenity::User) -> Result<(), Error> {
    let result = ctx.data().ckey.remove(&user.id).await?;

    ctx.send(|b| {
        b.ephemeral(true).content(if result {
            format!("{user} ckey mapping removed")
        } else {
            format!("{user} has no ckey mapping")
        })
    })
    .await?;

    Ok(())
}

/// Add user ckey to the whitelist
#[poise::command(
    context_menu_command = "Whitelist",
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn ckey_whitelist_ctx(ctx: Context<'_>, user: serenity::User) -> Result<(), Error> {
    let ckey = ctx.data().ckey.get_ckey(&user.id).await;

    match ckey {
        Some(ckey) => {
            let result = ctx.data().whitelist.insert(&ckey).await?;

            ctx.send(|b| {
                b.ephemeral(true).content(if result {
                    format!("{user} as `{ckey}` added to the whitelist")
                } else {
                    format!("{user} is already in the whitelist as `{ckey}`")
                })
            })
            .await?;
        }
        None => {
            ctx.send(|b| {
                b.ephemeral(true)
                    .content(format!("{user} has no ckey mapping"))
            })
            .await?;
        }
    }

    Ok(())
}

/// Remove user ckey from the whitelist
#[poise::command(
    context_menu_command = "Unwhitelist",
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn ckey_unwhitelist_ctx(ctx: Context<'_>, user: serenity::User) -> Result<(), Error> {
    let ckey = ctx.data().ckey.get_ckey(&user.id).await;

    match ckey {
        Some(ckey) => {
            let result = ctx.data().whitelist.remove(&ckey).await?;

            ctx.send(|b| {
                b.ephemeral(true).content(if result {
                    format!("{user} as `{ckey}` removed from the whitelist")
                } else {
                    format!("{user} as `{ckey}` is not in the whitelist")
                })
            })
            .await?;
        }
        None => {
            ctx.send(|b| {
                b.ephemeral(true)
                    .content(format!("{user} has no ckey mapping"))
            })
            .await?;
        }
    }

    Ok(())
}
