use crate::Context;

use std::env;

const MISSING_TEXT: &str = "Missing";
const POST_MISSING_TEXT: &str = "in your environment.";

pub fn check_env(key: &str) -> bool {
    env::var(key).is_ok()
}

pub fn parse_env_as_string(key: &str) -> String {
    env::var(key).expect(&format!("{} {} {}", MISSING_TEXT, key, POST_MISSING_TEXT)).parse().unwrap()
}

pub fn parse_env_as_u64(key: &str) -> u64 {
    env::var(key).expect(&format!("{} {} {}", MISSING_TEXT, key, POST_MISSING_TEXT)).parse().unwrap()
}

pub async fn is_staff(ctx: Context<'_>, user: &serenity::model::user::User) -> bool {
    user.has_role(ctx, parse_env_as_u64("PTL_GUILD_ID"), parse_env_as_u64("STAFF_ROLE_ID")).await.unwrap_or(false)
}

pub async fn is_qa(ctx: Context<'_>, user: &serenity::model::user::User) -> bool {
    user.has_role(ctx, parse_env_as_u64("PTL_GUILD_ID"), parse_env_as_u64("QA_ROLE_ID")).await.unwrap_or(false)
}