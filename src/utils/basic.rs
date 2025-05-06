use crate::Context;

pub async fn _is_staff(ctx: Context<'_>, user: &serenity::model::user::User) -> bool { // Currently not useful, access to commands will be configured within server settings.
    let config = &ctx.data().config;
    if user.id == config.owner { return true; }
    return false;
    // user.has_role(ctx, config.guild, config.roles.staff).await.unwrap_or(false)
}

pub async fn is_qa(ctx: Context<'_>, user: &serenity::model::user::User) -> bool {
    let config = &ctx.data().config;
    if user.id == config.owner { return true; }
    user.has_role(ctx, config.guild, config.roles.qa).await.unwrap_or(false)
}

pub fn generate_emoji_progressbar(current: u64, max: u64, progressbar_size: u64, progressbar_emojis: &crate::ProgressBarEmojisTypes) -> String {
    let fillstart = &progressbar_emojis.filled.start;
    let fillmid = &progressbar_emojis.filled.mid;
    let fillend = &progressbar_emojis.filled.end;
    
    let emptystart = &progressbar_emojis.empty.start;
    let emptymid = &progressbar_emojis.empty.mid;
    let emptyend = &progressbar_emojis.empty.end;

    let progress_float: f64 = (current as f64)/(max as f64)*(progressbar_size as f64);
    let progress: u64 = progress_float.floor() as u64;
    if progress >= 1 {
        if progress == 1 {
            return format!("\n{}{}{}",
                fillstart,
                emptymid.repeat((progressbar_size-2) as usize),
                emptyend
            );
        } else {
            if progressbar_size == progress {
                return format!("\n{}{}{}",
                    fillstart,
                    fillmid.repeat((progressbar_size-2) as usize),
                    fillend
                );
            } else {
                return format!("\n{}{}{}{}",
                    fillstart,
                    fillmid.repeat((progress-1) as usize),
                    emptymid.repeat((progressbar_size-progress-1) as usize),
                    emptyend
                );
            }
        }
    } else {
        if progress_float > 0.0 {
            return format!("\n{}{}{}",
                fillstart,
                emptymid.repeat((progressbar_size-2) as usize),
                emptyend
            );
        } else {
            return format!("\n{}{}{}",
                emptystart,
                emptymid.repeat((progressbar_size-2) as usize),
                emptyend
            );
        }
    }
}