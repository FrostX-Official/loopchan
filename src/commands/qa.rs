use crate::{Context, Error};

/// QA Managing Commands
#[poise::command(prefix_command, subcommands("status"), rename="QA")]
pub async fn qa(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Retrieve user's status (if they're in QA program or not.)
#[poise::command(slash_command)]
pub async fn status(
    ctx: Context<'_>,
    #[description = "Message"] user: Option<serenity::model::user::User>
) -> Result<(), Error> {
    if !user.is_none() {
        ctx.send(poise::CreateReply::default()
            .content(user.unwrap().id.to_string())
            .ephemeral(true)
        ).await?;
    } else {
        ctx.send(poise::CreateReply::default()
            .content(ctx.author().id.to_string())
            .ephemeral(true)
        ).await?;
    };

    Ok(())
}