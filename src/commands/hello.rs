use crate::{Context, Error};

/// Hello world!
#[poise::command(slash_command)]
pub async fn hello(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(poise::CreateReply::default()
        .content("yo")
        .ephemeral(true)
    ).await?;
    Ok(())
}