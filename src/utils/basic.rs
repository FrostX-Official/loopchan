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

pub fn generate_emoji_progressbar(current: u64, max: u64, progressbar_size: u64) -> String {
    let progress_float: f64 = (current as f64)/(max as f64)*(progressbar_size as f64);
    let progress: u64 = progress_float.floor() as u64;
    let progressbar: String;
    if progress >= 1 {
        if progress == 1 {
            progressbar = format!("\n{}{}{}",
                "<:LoopchanProgressbarFillStart:1368315375048986817>",
                "<:LoopchanProgressbarProgress:1368315376450146426>".repeat((progressbar_size-2) as usize),
                "<:LoopchanProgressbarEnd:1368315369722351617>"
            );
        } else {
            progressbar = format!("\n{}{}{}{}",
                "<:LoopchanProgressbarFillStart:1368315375048986817>",
                "<:LoopchanProgressbarFillProgress:1368315373547683980>".repeat((progress-1) as usize),
                "<:LoopchanProgressbarProgress:1368315376450146426>".repeat((progressbar_size-progress-1) as usize),
                "<:LoopchanProgressbarEnd:1368315369722351617>"
            );
        }
    } else {
        if progress_float > 0.0 {
            progressbar = format!("\n{}{}{}",
                "<:LoopchanProgressbarFillStart:1368315375048986817>",
                "<:LoopchanProgressbarProgress:1368315376450146426>".repeat((progressbar_size-2) as usize),
                "<:LoopchanProgressbarEnd:1368315369722351617>"
            );
        } else {
            progressbar = format!("\n{}{}{}",
                "<:LoopchanProgressbarStart:1368315378404429914>",
                "<:LoopchanProgressbarProgress:1368315376450146426>".repeat((progressbar_size-2) as usize),
                "<:LoopchanProgressbarEnd:1368315369722351617>"
            );
        }
    }
    return progressbar;
}