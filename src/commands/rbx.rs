use roboat::thumbnails::{ThumbnailSize, ThumbnailType};
use serenity::all::{Colour, CreateEmbed};

use crate::{Context, Error};

/// Fetches roblox's account data
#[poise::command(slash_command)]
pub async fn fetchdata(
    ctx: Context<'_>,
    #[description = "Roblox User ID"] roblox_user_id: u64,
) -> Result<(), Error> {
    let roblox_client: &roboat::Client = &ctx.data().roblox_client;

    let roblox_user: Result<roboat::users::UserDetails, roboat::RoboatError> = roblox_client.user_details(roblox_user_id).await;
    if roblox_user.is_ok() {
        let roblox_user = roblox_user.unwrap();
        let headshot_thubmnail = roblox_client.thumbnail_url(roblox_user.id, ThumbnailSize::S420x420, ThumbnailType::AvatarHeadshot).await;
        if headshot_thubmnail.is_ok() {
            ctx.send(poise::CreateReply::default()
                .embed(
                    CreateEmbed::default()
                        .title("✅ User Found!")
                        .image(headshot_thubmnail.unwrap())
                        .field("User ID", roblox_user.id.to_string(), false)
                        .field("Display Name", roblox_user.display_name, false)
                        .field("Username", roblox_user.username, false)
                        .color(Colour::from_rgb(80, 255, 80))
                )
                //.ephemeral(true)  
            ).await?;
        } else {
            ctx.send(poise::CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .title("✅ User Found!")
                    .field("User ID", roblox_user.id.to_string(), false)
                    .field("Display Name", roblox_user.display_name, false)
                    .field("Username", roblox_user.username, false)
                    .color(Colour::from_rgb(80, 255, 80))
            )
            //.ephemeral(true)
        ).await?;
        }
    } else {
        ctx.send(poise::CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .title("❌ User Not Found!")
                    .color(Colour::from_rgb(255, 80, 80))
            )
            .ephemeral(true)
        ).await?;
    }
    Ok(())
}