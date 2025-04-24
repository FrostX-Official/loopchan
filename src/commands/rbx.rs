use crate::{Context, Error};

/// Fetches roblox's account data
#[poise::command(slash_command)]
pub async fn fetchdata(ctx: Context<'_>) -> Result<(), Error> {
    let roblox_client: &roboat::Client = &ctx.data().roblox_client;

    let frostxoff: Result<roboat::users::UserDetails, roboat::RoboatError> = roblox_client.user_details(1631662564).await;
    if frostxoff.is_ok() {
        ctx.send(poise::CreateReply::default()
            .content(frostxoff.unwrap().username)
            .ephemeral(true)
        ).await?;
    } else {
        ctx.send(poise::CreateReply::default()
            .content("no")
            .ephemeral(true)
        ).await?;
    }
    Ok(())
}