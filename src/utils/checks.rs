use crate::Context;

pub async fn is_staff(ctx: Context<'_>, user: &serenity::model::user::User) -> bool {
    let custom_data = ctx.data();
    user.has_role(ctx, custom_data.guild_id, custom_data.staff_role_id).await.unwrap_or(false)
}

pub async fn is_qa(ctx: Context<'_>, user: &serenity::model::user::User) -> bool {
    let custom_data = ctx.data();
    user.has_role(ctx, custom_data.guild_id, custom_data.qa_role_id).await.unwrap_or(false)
}