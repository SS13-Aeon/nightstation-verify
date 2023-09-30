use crate::app::{models::Ckey, Context, Error};

/// Modify and inspect whitelist
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR",
    subcommands("reload", "list", "add", "remove")
)]
pub async fn whitelist(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(|b| b.ephemeral(true).content("Use one of the subcommands"))
        .await?;
    Ok(())
}

/// Reload the whitelist from disk
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn reload(ctx: Context<'_>) -> Result<(), Error> {
    ctx.data().whitelist.load().await?;

    ctx.send(|b| b.ephemeral(true).content("Whitelist reloaded from disk"))
        .await?;

    Ok(())
}

/// Show the entire whitelist
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let list: Vec<_> = ctx
        .data()
        .whitelist
        .list()
        .await
        .chunks(20)
        .map(|chunk| {
            chunk
                .iter()
                .map(|ckey| format!("`{ckey}`"))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect();

    if list.is_empty() {
        ctx.send(|b| b.ephemeral(true).content("Whitelist is empty"))
            .await?;

        return Ok(());
    }

    poise::samples::paginate(ctx, &list.iter().map(|s| &s[..]).collect::<Vec<_>>()).await?;

    Ok(())
}

/// Add ckey to the whitelist
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Ckey to add to the whitelist"] ckey: String,
) -> Result<(), Error> {
    let ckey = Ckey::from(&ckey);
    let result = ctx.data().whitelist.insert(&ckey).await?;

    ctx.send(|b| {
        b.ephemeral(true).content(if result {
            format!("`{ckey}` added to the whitelist")
        } else {
            format!("`{ckey}` is already in the whitelist")
        })
    })
    .await?;

    Ok(())
}

/// Remove ckey from the whitelist
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "Ckey to remove from the whitelist"] ckey: String,
) -> Result<(), Error> {
    let ckey = Ckey::from(&ckey);
    let result = ctx.data().whitelist.remove(&ckey).await?;

    ctx.send(|b| {
        b.ephemeral(true).content(if result {
            format!("`{ckey}` removed from the whitelist")
        } else {
            format!("`{ckey}` is not in the whitelist")
        })
    })
    .await?;

    Ok(())
}
