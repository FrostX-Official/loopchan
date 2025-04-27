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
    let guild_id: u64 = custom_data.guild_id;
    let staff_role_id: u64 = custom_data.staff_role_id;
    let qa_role_id: u64 = custom_data.qa_role_id;

    let nuser: serenity::model::user::User;

    if user.is_none() {
        nuser = ctx.author().clone();
    } else {
        nuser = user.unwrap();
    }

    let is_staff: bool = nuser.has_role(ctx, guild_id, staff_role_id).await.unwrap_or(false);
    let is_qa: bool = nuser.has_role(ctx, guild_id, qa_role_id).await.unwrap_or(false);
    db::create_user_in_db(&custom_data.db_client, nuser.id.into(), 0, is_staff.clone(), is_qa.clone()).await?;

    ctx.send(poise::CreateReply::default()
        .content(is_qa.to_string())
        .ephemeral(true)
    ).await?;
 
    Ok(())
}