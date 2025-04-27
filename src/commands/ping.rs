use crate::{Context, Error};

/// Check bot latency
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let ping_duration: std::time::Duration = ctx.ping().await;
    let mut ping_duration_str: String = ping_duration.as_millis().to_string()+"ms";
    if ping_duration_str == "0ms" {
        ping_duration_str = String::from("Loading...");
    }
    ctx.send(poise::CreateReply::default()
        .content("hiii!! hewwoo hiiii! :3\n-# Latency: ".to_owned()+&ping_duration_str) // what is this T-T
        .ephemeral(true)
    ).await?;
    Ok(())
}