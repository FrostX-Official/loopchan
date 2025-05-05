use serenity::all::{ButtonStyle, Color, CreateActionRow, CreateButton, CreateEmbed, CreateMessage};

use crate::{Context, Error};
use tracing::error;

/// Bot Debug Commands
#[poise::command(slash_command, subcommands("ping", "register", "wordgen", "postverificationchannellink", "clearlogs"), subcommand_required)]
pub async fn debug(_ctx: Context<'_>) -> Result<(), Error> { Ok(()) }

/// Check bot latency
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let ping_duration: std::time::Duration = ctx.ping().await;
    let mut ping_duration_str: String = ping_duration.as_millis().to_string()+"ms";
    if ping_duration_str == "0ms" {
        ping_duration_str = String::from("Loading...");
    }
    ctx.send(poise::CreateReply::default()
        .content(format!("hiii!! hewwoo hiiii! :3\n-# Latency: {}", ping_duration_str)) // what is this T-T
        //.ephemeral(true)
    ).await?;
    Ok(())
}

/// Generate random word from wordgen module
#[poise::command(slash_command)]
pub async fn wordgen(
    ctx: Context<'_>,
    #[min_length = 1] #[max_length = 254] #[description = "Amount"] amount: u8
) -> Result<(), Error> {
    let mut randomwords: Vec<String> = vec![];
    for _ in 0..amount+1 {
        randomwords.insert(0, crate::utils::wordgen::getrandomgenword().await);
    }
    ctx.send(poise::CreateReply::default()
        .content(format!("```{}```", randomwords.join("\n")))
        .ephemeral(true)
    ).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn clearlogs(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let log_file = &ctx.data().log_file;
    let logs_read_successful: Result<tokio::fs::ReadDir, std::io::Error> = tokio::fs::read_dir("logs").await;

    if logs_read_successful.is_err() {
        error!("Failed to read logs: {}", logs_read_successful.unwrap_err().to_string());
        ctx.send(poise::CreateReply::default()
            .content("Failed to read logs. Check console.")
            .ephemeral(true)
        ).await?;
        return Ok(());
    }

    let mut logs_read: tokio::fs::ReadDir = logs_read_successful.unwrap();

    while let Some(entry) = logs_read.next_entry().await? {
        let path: std::path::PathBuf = entry.path();
        if path.to_str().unwrap() == "logs\\".to_owned()+log_file {
            continue;
        }
        let is_successful: Result<(), std::io::Error> = tokio::fs::remove_file(&path).await;
        if is_successful.is_err() {
            error!("Failed to delete log \"{}\": {}", path.to_str().unwrap(), is_successful.unwrap_err().to_string());
            ctx.send(poise::CreateReply::default()
                .content("Failed to delete logs. Check console.")
                .ephemeral(true)
            ).await?;
            return Ok(());
        }
    }

    ctx.send(poise::CreateReply::default()
        .content("Successful!")
        .ephemeral(true)
    ).await?;

    Ok(())
}

/// Generate random word from wordgen module
#[poise::command(slash_command)]
pub async fn postverificationchannellink(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let loopchans_config = &ctx.data().config;
    ctx.channel_id().send_message(ctx, CreateMessage::default()
        .add_embed(
            CreateEmbed::default()
                .description(format!("Type slash command: `/verify` with your Roblox Username **or** ID\nin <#{}> to proceed!", loopchans_config.channels.unverified_chat))
                .image("https://media.discordapp.net/attachments/1193463119532527646/1364308828195127366/PaJbaO4.png?ex=68170adc&is=6815b95c&hm=76a917c2e090636ba42d90299301f6d5dd99fa2f733b222c2ad6cc0131a3f186&=&width=1872&height=624")
                .color(Color::from_rgb(255, 255, 255))
        )
        .components(vec![ // nesting hell
            CreateActionRow::Buttons(vec![
                CreateButton::new_link(format!("https://discord.com/channels/{}/{}", loopchans_config.guild, loopchans_config.channels.unverified_chat))
                    .label("Go to Unverified Chat")
                    .style(ButtonStyle::Secondary)
                    .emoji('💬'),
        ])])
    ).await?;

    ctx.send(poise::CreateReply::default()
        .content("sent")
        .ephemeral(true)
    ).await?;

    Ok(())
}

/// Slash Commands Registering Handler
#[poise::command(slash_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}