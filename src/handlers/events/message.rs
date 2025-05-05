use std::time::Duration;
use chrono::Datelike;

use ::serenity::all::Message;

use tokio::time::Instant;
use tracing::info;

pub async fn give_exp_for_message(
    message: &Message,
    data: &crate::Data
) {
    if message.author.bot { return; }
    if message.content.len() < 2 { return; }
    let userid: u64 = message.author.id.get();
    let cooldown_duration: Duration = Duration::from_secs(10);
    let mut cooldowns = data.exp_cooldowns.lock().await;
    let last_exp_time = cooldowns.entry(userid).or_insert((Instant::now() - cooldown_duration).into());

    if last_exp_time.elapsed() >= cooldown_duration {
        let leveling_config = &data.config.leveling;
        let mut weekday_multiplier: u64 = 1;
        if leveling_config.double_multiplier_on_weekdays {
            let weekday: chrono::Weekday = chrono::offset::Local::now().date_naive().weekday();
            if weekday == chrono::Weekday::Sat {
                weekday_multiplier = 2
            } else if weekday == chrono::Weekday::Sun {
                weekday_multiplier = 2
            }
        }
        let exp_amount: u64 = message.content.len().min(leveling_config.max_exp_per_message as usize) as u64*leveling_config.exp_multiplier*weekday_multiplier;
        crate::commands::eco::give_user_eco_exp(data, &message.author, exp_amount).await;
        *last_exp_time = Instant::now().into();
    } else {
        info!("Tried to give {} exp after message, but it's on cooldown", userid)
    }
}