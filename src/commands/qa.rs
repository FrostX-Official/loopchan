use crate::utils::checks::is_qa;
use crate::{Context, Error};
use crate::utils::db;

/// QA Managing Commands
#[poise::command(slash_command, subcommands("status"), subcommand_required)]
pub async fn qa(_ctx: Context<'_>) -> Result<(), Error> { Ok(()) }

/// Retrieve user's status (if they're in QA program or not.)
#[poise::command(slash_command)]
pub async fn status(
    ctx: Context<'_>,
    #[description = "Message"] user: Option<serenity::model::user::User>
) -> Result<(), Error> {
    let custom_data = ctx.data();

    let nuser: serenity::model::user::User;

    if user.is_none() {
        nuser = ctx.author().clone();
    } else {
        nuser = user.unwrap();
    }

    let is_qa_v: bool = is_qa(ctx, &nuser).await;
    db::create_user_in_db(&custom_data.db_client, nuser.id.into(), 0).await?;

    ctx.send(poise::CreateReply::default()
        .content(is_qa_v.to_string())
        .ephemeral(true)
    ).await?;
 
    Ok(())
}