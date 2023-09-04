use poise::serenity_prelude as serenity;

use crate::app::{
    services::{SendGreetingError, VerificationError, VerificationStatus},
    Context, Error,
};

/// Modify and inspect verification data
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR",
    subcommands("status", "clear", "greet")
)]
pub async fn verification(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(|b| b.ephemeral(true).content("Use one of the subcommands"))
        .await?;
    Ok(())
}

/// Get member verification status
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn status(ctx: Context<'_>, member: serenity::Member) -> Result<(), Error> {
    let response = match ctx.data().verification.get_status(&member.user.id).await {
        None => "No data",
        Some(status) => match status {
            VerificationStatus::Greeted { greeting_id: _ } => "Greeted",
            VerificationStatus::Pending {
                form_data: _,
                form_idx: _,
            } => "Pending",
            VerificationStatus::Rejected => "Rejected",
            VerificationStatus::Verified => "Verified",
        },
    };

    ctx.send(|b| b.ephemeral(true).content(response)).await?;

    Ok(())
}

/// Get member verification status
#[poise::command(
    context_menu_command = "Verification status",
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn verification_status_ctx(ctx: Context<'_>, user: serenity::User) -> Result<(), Error> {
    let response = match ctx.data().verification.get_status(&user.id).await {
        None => "No data",
        Some(status) => match status {
            VerificationStatus::Greeted { greeting_id: _ } => "Greeted",
            VerificationStatus::Pending {
                form_data: _,
                form_idx: _,
            } => "Pending",
            VerificationStatus::Rejected => "Rejected",
            VerificationStatus::Verified => "Verified",
        },
    };

    ctx.send(|b| b.ephemeral(true).content(response)).await?;

    Ok(())
}

/// Clear member verification
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn clear(ctx: Context<'_>, member: serenity::Member) -> Result<(), Error> {
    let response = ctx.data().verification.remove(&member.user.id).await?;

    if response {
        ctx.send(|b| {
            b.ephemeral(true)
                .content(format!("Cleared verification data for {member}"))
        })
        .await?;
    } else {
        ctx.send(|b| {
            b.ephemeral(true)
                .content(format!("{member} has no verification data"))
        })
        .await?;
    }

    Ok(())
}

/// Clear member verification
#[poise::command(
    context_menu_command = "Clear verification",
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn verification_clear_ctx(ctx: Context<'_>, user: serenity::User) -> Result<(), Error> {
    let response = ctx.data().verification.remove(&user.id).await?;

    if response {
        ctx.send(|b| {
            b.ephemeral(true)
                .content(format!("Cleared verification data for {user}"))
        })
        .await?;
    } else {
        ctx.send(|b| {
            b.ephemeral(true)
                .content(format!("{user} has no verification data"))
        })
        .await?;
    }

    Ok(())
}

/// Send greeting to member
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn greet(ctx: Context<'_>, member: serenity::Member) -> Result<(), Error> {
    let response = ctx
        .data()
        .verification
        .send_greeting(ctx.serenity_context(), &member.user)
        .await;

    let response = match response {
        Ok(_) => "Greeting sent",
        Err(VerificationError::SendGreeting(e)) => match e {
            SendGreetingError::AlreadyPending => "Already pending",
            SendGreetingError::AlreadyVerified => "Already verified",
            SendGreetingError::AlreadyRejected => "Already rejected",
            e @ SendGreetingError::Discord(_) => {
                return Err(VerificationError::SendGreeting(e).into())
            }
        },
        Err(e) => return Err(e.into()),
    };

    ctx.send(|b| b.ephemeral(true).content(response)).await?;

    Ok(())
}

/// Send greeting to member
#[poise::command(
    context_menu_command = "Greet",
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn verification_greet_ctx(ctx: Context<'_>, user: serenity::User) -> Result<(), Error> {
    let response = ctx
        .data()
        .verification
        .send_greeting(ctx.serenity_context(), &user)
        .await;

    let response = match response {
        Ok(_) => "Greeting sent",
        Err(VerificationError::SendGreeting(e)) => match e {
            SendGreetingError::AlreadyPending => "Already pending",
            SendGreetingError::AlreadyVerified => "Already verified",
            SendGreetingError::AlreadyRejected => "Already rejected",
            e @ SendGreetingError::Discord(_) => {
                return Err(VerificationError::SendGreeting(e).into())
            }
        },
        Err(e) => return Err(e.into()),
    };

    ctx.send(|b| b.ephemeral(true).content(response)).await?;

    Ok(())
}
