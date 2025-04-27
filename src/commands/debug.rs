use crate::{utils::checks::is_staff, Context, Error};

/// Bot Debug Commands
#[poise::command(slash_command, subcommands("ping", "register", "wordgen"), subcommand_required)]
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
        .content("hiii!! hewwoo hiiii! :3\n-# Latency: ".to_owned()+&ping_duration_str) // what is this T-T
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
    if !is_staff(ctx, ctx.author()).await {
        ctx.send(poise::CreateReply::default()
            .content("No Access.")
            //.ephemeral(true)
        ).await?;
        return Ok(());
    }

    let mut randomwords: Vec<String> = vec![];
    for _ in 0..amount+1 {
        randomwords.insert(0, crate::utils::wordgen::getrandomgenword().await);
    }
    ctx.send(poise::CreateReply::default()
        .content("```".to_owned()+&randomwords.join("
")+"```")
        //.ephemeral(true)
    ).await?; 
    Ok(())
}

/// Slash Commands Registering Handler
#[poise::command(slash_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    if !is_staff(ctx, ctx.author()).await {
        ctx.send(poise::CreateReply::default()
            .content("No Access.")
            //.ephemeral(true)
        ).await?;
        return Ok(());
    }

    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}