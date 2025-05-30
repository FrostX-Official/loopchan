use ::serenity::all::{Member, CreateMessage, CreateEmbed, Color, Message, Error};

use poise::serenity_prelude as serenity;
use tracing::{error, warn};

pub async fn blacklist_check(
    ctx: &serenity::Context,
    new_member: &Member,
    data: &crate::Data
) -> Result<bool, Error> {
    let blacklist: &Vec<u64> = &data.config.blacklist;
    if blacklist.contains(&new_member.user.id.get()) {
        let sent_message: Result<Message, Error> = new_member.user.direct_message(ctx,
            CreateMessage::default()
                .embed(
                    CreateEmbed::default()
                        .description("Hello! You have been blacklisted from **PARKOUR: The Loop** before server launch.\nThis means that you've been permanently banned from **datalose** earlier, without appealing.\nCurrently appealing a blacklist is not possible. However, we will let you know if that changes ever.")
                        .color(Color::from_rgb(255, 0, 0))
                )
        ).await;

        if sent_message.is_err() {
            warn!("Failed to send blacklist message to {}: {}", new_member.user.id.get(), sent_message.unwrap_err().to_string());
        } else {
            sent_message.unwrap().reply(ctx, "You can run command ```/blacklist_check``` in loopchan's DMs to get your blacklist status and appeal server once it becomes available.").await?;
        }

        let is_banned: Result<(), serenity::Error> = new_member.ban_with_reason(ctx, 7, "Blacklisted UserId").await;

        if is_banned.is_err() {
            let err_unwrapped = is_banned.unwrap_err();
            error!("Failed to ban {}: {} (blacklist ban)", new_member.user.id.get(), err_unwrapped.to_string());
            return Err(err_unwrapped);
        }

        return Ok(true);
    }

    return Ok(false);
}