use serenity::all::{Color, CreateEmbed, CreateMessage, Member, Message};
use tracing::error;

use crate::{Context, Error, LoopchanConfig};

/// Bot ADM Commands
#[poise::command(slash_command, subcommands("blacklist"), subcommand_required)]
pub async fn adm(_ctx: Context<'_>) -> Result<(), Error> { Ok(()) }

/// Blacklisting LIVE
#[poise::command(slash_command)]
pub async fn blacklist(
    ctx: Context<'_>,
    #[description = "Member to blacklist"]
    member: Member,
) -> Result<(), Error> {
    let toml_string: Result<String, std::io::Error> = tokio::fs::read_to_string("Config.toml").await;
    if toml_string.is_err() {
        error!("Failed to read your config.toml file: {}", toml_string.unwrap_err().to_string());
        return Ok(());
    }

    let mut loopchans_config: LoopchanConfig = toml::from_str(&toml_string.unwrap()).unwrap();
    loopchans_config.blacklist.push(member.user.id.get());

    let written: Result<(), std::io::Error> = tokio::fs::write("Config.toml", toml::to_string(&loopchans_config).unwrap()).await;

    if written.is_err() {
        error!("{}", written.err().unwrap().to_string());
        ctx.send(poise::CreateReply::default()
            .content("Failed to write new toml. Check terminal logs.")
            .ephemeral(true)
        ).await?;
        return Ok(());
    }

    let sent_message: Result<Message, serenity::Error> = member.user.direct_message(ctx,
        CreateMessage::default()
            .embed(
                CreateEmbed::default()
                    .description("Hello! You have been blacklisted from **PARKOUR: The Loop**.\nThis means that you will not be able to access any of the content related to **PTL**, or it's community.\nReasons of blacklisting are not being disclosed, if you were blacklisted this means that you should know what you did.\nCurrently appealing a blacklist is not possible. However, we will let you know if that changes ever.")
                    .color(Color::from_rgb(255, 0, 0))
            )
    ).await;

    if sent_message.is_err() {
        error!("Failed to send blacklist message to {}: {}", member.user.id.get(), sent_message.unwrap_err().to_string());
    } else {
        sent_message.unwrap().reply(ctx, "You can run command ```/blacklist_check``` in loopchan's DMs to get your blacklist status and appeal server once it becomes available.").await?;
    }

    let banned: Result<(), serenity::Error> = member.ban_with_reason(ctx, 0, "Blacklisted UserId (LIVE)").await;

    if banned.is_err() {
        error!("{}", banned.err().unwrap().to_string());
        ctx.send(poise::CreateReply::default()
            .content("Failed to ban blacklisted user. Check terminal logs.")
            .ephemeral(true)
        ).await?;
        return Ok(());
    }

    ctx.send(poise::CreateReply::default()
        .content("Successfully blacklisted.")
        .ephemeral(true)
    ).await?;
    
    Ok(())
}