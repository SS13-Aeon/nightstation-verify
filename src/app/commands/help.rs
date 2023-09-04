use crate::{
    app::{Context, Error},
    COPYRIGHT, SOURCE,
};

/// Show this menu
#[poise::command(slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> Result<(), Error> {
    let config = poise::builtins::HelpConfiguration {
        show_context_menu_commands: true,
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

/// Show version
#[poise::command(slash_command)]
pub async fn version(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(|b| {
        b.ephemeral(true)
            .content(format!("`{}`", clap::crate_version!()))
    })
    .await?;

    Ok(())
}

/// Show license
#[poise::command(slash_command)]
pub async fn license(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(|b| {
        b.ephemeral(true)
            .content(format!("```txt\n{COPYRIGHT}\n```"))
    })
    .await?;

    Ok(())
}

/// Show source code link
#[poise::command(slash_command)]
pub async fn source(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(|b| {
        b.ephemeral(true).content(format!(
            "Copies of the source code can be obtained from {SOURCE}"
        ))
    })
    .await?;

    Ok(())
}
