use crate::Context;

const MISSING_TEXT: &str = "missing ";

pub fn parse_env_as_string(key: &str) -> String {
    std::env::var(key).expect(&(MISSING_TEXT.to_owned()+key)).parse().unwrap()
}

pub fn parse_env_as_u64(key: &str) -> u64 {
    std::env::var(key).expect(&(MISSING_TEXT.to_owned()+key)).parse().unwrap()
}

pub async fn is_staff(ctx: Context<'_>, user: &serenity::model::user::User) -> bool {
    user.has_role(ctx, parse_env_as_u64("PTL_GUILD_ID"), parse_env_as_u64("STAFF_ROLE_ID")).await.unwrap_or(false)
}

pub async fn is_qa(ctx: Context<'_>, user: &serenity::model::user::User) -> bool {
    user.has_role(ctx, parse_env_as_u64("PTL_GUILD_ID"), parse_env_as_u64("QA_ROLE_ID")).await.unwrap_or(false)
}