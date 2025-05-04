use crate::Context;

pub async fn is_staff(ctx: Context<'_>, user: &serenity::model::user::User) -> bool {
    let config = &ctx.data().config;
    if user.id == config.owner { return true; }
    user.has_role(ctx, config.guild, config.roles.staff).await.unwrap_or(false)
}

pub async fn is_qa(ctx: Context<'_>, user: &serenity::model::user::User) -> bool {
    let config = &ctx.data().config;
    if user.id == config.owner { return true; }
    user.has_role(ctx, config.guild, config.roles.qa).await.unwrap_or(false)
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