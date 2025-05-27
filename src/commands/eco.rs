use std::{time::Duration, vec};

use crate::{utils::{basic::generate_emoji_progressbar, database::economy::increment_user_balance_in_eco_db}, Context, Error, RoleShopItem};

use poise::{CooldownConfig, CreateReply};
use rand::Rng;
use serenity::all::{Color, CreateActionRow, CreateEmbed, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption, ReactionType};
use tracing::{error, info};

use crate::utils::database::economy::{
    create_user_in_eco_db,
    get_user_balance_in_eco_db,
    get_user_level_and_experience_in_eco_db,
    update_user_level_and_experience_in_eco_db,
    build_balance_leaderboard_from_eco_db,
    build_level_leaderboard_from_eco_db,
    get_user_placement_in_balance_leaderboard,
    get_user_placement_in_level_leaderboard
};

fn exp_needed_to_next_level(current_level: u64) -> u64 {
    let level: f64 = current_level as f64;
    return (5.0 * (level.powf(2.0)) + (50.0 * level) + 100.0).ceil() as u64;
}

pub async fn handle_user_exp_update(
    db_client: &async_sqlite::Client, // db client to index economics in
    userid: u64, // User ID to index in economics
    level: u64, // User's level before adding experience
    experience: u64, // New experience (not the one in db)
) -> Result<usize, async_sqlite::Error> {
    let mut level: u64 = level;
    let mut experience: u64 = experience;
    let mut experience_needed: u64 = exp_needed_to_next_level(level);

    if experience_needed <= experience {
        loop {
            if experience_needed > experience {
                break
            }
            experience -= experience_needed;
            level += 1;
            experience_needed = exp_needed_to_next_level(level);
            info!("{} leveled up! ({} lvl now, experience: {}/{})", userid, level, experience, experience_needed);
        }
    }

    update_user_level_and_experience_in_eco_db(db_client, userid, Some(level), Some(experience)).await
}

pub async fn give_user_eco_exp(
    custom_data: &crate::Data,
    user: &serenity::model::user::User,
    amount: u64
) {
    let userid: u64 = user.id.get();
    let successfully_created: Result<usize, async_sqlite::Error> = create_user_in_eco_db(&custom_data.db_client, userid).await; // Why can't I put ? here to ignore Result???
    if successfully_created.is_err() {
        error!("Failed to create user ({}) in eco db: {}", userid, successfully_created.unwrap_err().to_string());
        return;
    }

    let level_exp_check: Result<(Result<u64, async_sqlite::rusqlite::Error>, Result<u64, async_sqlite::rusqlite::Error>), async_sqlite::Error> = get_user_level_and_experience_in_eco_db(&custom_data.db_client, userid).await;
    
    if !level_exp_check.is_ok() {
        error!("Failed to check {}'s level and experience: {}", userid, level_exp_check.unwrap_err().to_string());
        return;
    }

    let level_and_exp_checks: (Result<u64, async_sqlite::rusqlite::Error>, Result<u64, async_sqlite::rusqlite::Error>) = level_exp_check.unwrap();

    if !level_and_exp_checks.0.is_ok() {
        error!("Failed to check {}'s level: {}", userid, level_and_exp_checks.0.unwrap_err().to_string());
        return;
    }

    if !level_and_exp_checks.1.is_ok() {
        error!("Failed to check {}'s experience: {}", userid, level_and_exp_checks.1.unwrap_err().to_string());
        return;
    }

    let level: u64 = level_and_exp_checks.0.unwrap();
    let mut experience: u64 = level_and_exp_checks.1.unwrap();

    experience += amount;

    let successfully_updated: Result<usize, async_sqlite::Error> = handle_user_exp_update(&custom_data.db_client, userid, level, experience).await;
    if successfully_updated.is_err() {
        error!("Failed to update user ({}) in eco db: {}", userid, successfully_updated.unwrap_err().to_string());
    }
}

/// Economics Commands
#[poise::command(slash_command, subcommands("balance", "level", "modify_data", "leaderboard", "roleshop", "work"), subcommand_required)]
pub async fn eco(_ctx: Context<'_>) -> Result<(), Error> { Ok(()) }

#[poise::command(slash_command)]
pub async fn modify_data(
    ctx: Context<'_>,
    #[description = "Member"] user: Option<serenity::model::user::User>,
    #[description = "Level"] level: Option<u64>,
    #[description = "Experience"] experience: Option<u64>
) -> Result<(), Error> {
    if level.is_none() && experience.is_none() {
        ctx.send(CreateReply::default()
            .content("Provide level and/or experience.")
            .ephemeral(true)
        ).await?;
        return Ok(());
    }

    let db_client: &async_sqlite::Client = &ctx.data().db_client;

    let nuser: &serenity::model::user::User = if user.is_none() {
        ctx.author()
    } else {
        &user.unwrap()
    };
    let nuser_id: u64 = nuser.id.get();
    
    let s = create_user_in_eco_db(db_client, nuser_id).await;
    if s.is_err() {
        error!("Failed to create user in db: {}", s.unwrap_err().to_string());
        ctx.send(CreateReply::default()
            .content("Failed to create user in db! (check console~)")
            .ephemeral(true)
        ).await?;
        return Ok(());
    }

    let successful: Result<usize, async_sqlite::Error>;
    if experience.is_some() {
        let actual_level: u64;

        if level.is_some() {
            actual_level = level.unwrap()
        } else {
            let level_exp_check: Result<(Result<u64, async_sqlite::rusqlite::Error>, Result<u64, async_sqlite::rusqlite::Error>), async_sqlite::Error> = get_user_level_and_experience_in_eco_db(db_client, nuser_id).await;
        
            if level_exp_check.is_err() {
                error!("Failed to check {}'s level and experience: {}", nuser_id, level_exp_check.unwrap_err().to_string());
                ctx.send(CreateReply::default()
                    .content("Failed to modify data! (check console~)")
                    .ephemeral(true)
                ).await?;
                return Ok(());
            }

            let level_and_exp_checks: (Result<u64, async_sqlite::rusqlite::Error>, Result<u64, async_sqlite::rusqlite::Error>) = level_exp_check.unwrap();

            if level_and_exp_checks.0.is_err() {
                error!("Failed to check {}'s level: {}", nuser_id, level_and_exp_checks.0.unwrap_err().to_string());
                ctx.send(CreateReply::default()
                    .content("Failed to modify data! (check console~)")
                    .ephemeral(true)
                ).await?;
                return Ok(());
            }

            actual_level = level_and_exp_checks.0.unwrap();
        }

        successful = handle_user_exp_update(db_client, nuser_id, actual_level, experience.unwrap()).await;
    } else {
        successful = update_user_level_and_experience_in_eco_db(db_client, nuser_id, level, experience).await;
    }

    if successful.is_ok() {
        ctx.send(CreateReply::default()
            .content(format!("Successful\n-# usize: {}", successful.unwrap()))
        ).await?;
    } else {
        error!("Failed to modify data: {}", successful.err().unwrap().to_string());
        ctx.send(CreateReply::default()
            .content("Failed to modify data! (check console~)")
            .ephemeral(true)
        ).await?;
    }

    Ok(())
}

/// Check your balance or balance of other member
#[poise::command(slash_command)]
pub async fn balance(
    ctx: Context<'_>,
    #[description = "Member"] user: Option<serenity::model::user::User>
) -> Result<(), Error> {
    let custom_data = ctx.data();

    let nuser: &serenity::model::user::User = if user.is_none() {
        ctx.author()
    } else {
        &user.unwrap()
    };

    let nuser_id: u64 = nuser.id.into();

    create_user_in_eco_db(&custom_data.db_client, nuser_id).await?;
    let balance_check: Result<u64, async_sqlite::Error> = get_user_balance_in_eco_db(&custom_data.db_client, nuser_id).await;
    
    if !balance_check.is_ok() {
        error!("Failed to check {}'s balance: {}", nuser_id, balance_check.unwrap_err().to_string());
        ctx.send(CreateReply::default()
            .content("Failed to check user's balance! Please try again later or report this issue to <@908779319084589067>.")
            .ephemeral(true)
        ).await?;

        return Ok(());
    }

    let balance: u64 = balance_check.unwrap();

    ctx.send(CreateReply::default()
        .content(format!("<@{}>'s Balance: {}", nuser.id, balance))
        .ephemeral(true)
    ).await?;

    Ok(())
}

/// Check your level or level of other member
#[poise::command(slash_command)]
pub async fn level(
    ctx: Context<'_>,
    #[description = "Member"] user: Option<serenity::model::user::User>
) -> Result<(), Error> {
    let custom_data = ctx.data();

    let nuser: &serenity::model::user::User = if user.is_none() {
        ctx.author()
    } else {
        &user.unwrap()
    };

    let nuser_id: u64 = nuser.id.into();

    create_user_in_eco_db(&custom_data.db_client, nuser_id).await?;
    let level_exp_check: Result<(Result<u64, async_sqlite::rusqlite::Error>, Result<u64, async_sqlite::rusqlite::Error>), async_sqlite::Error> = get_user_level_and_experience_in_eco_db(&custom_data.db_client, nuser_id).await;
    
    if !level_exp_check.is_ok() {
        error!("Failed to check {}'s level and experience: {}", nuser_id, level_exp_check.unwrap_err().to_string());
        ctx.send(CreateReply::default()
            .content("Failed to check user's level! Please try again later or report this issue to <@908779319084589067>.")
            .ephemeral(true)
        ).await?;

        return Ok(());
    }

    let level_and_exp_checks: (Result<u64, async_sqlite::rusqlite::Error>, Result<u64, async_sqlite::rusqlite::Error>) = level_exp_check.unwrap();

    if !level_and_exp_checks.0.is_ok() {
        error!("Failed to check {}'s level: {}", nuser_id, level_and_exp_checks.0.unwrap_err().to_string());
        ctx.send(CreateReply::default()
            .content("Failed to check user's level! Please try again later or report this issue to <@908779319084589067>.")
            .ephemeral(true)
        ).await?;

        return Ok(());
    }

    if !level_and_exp_checks.1.is_ok() {
        error!("Failed to check {}'s experience: {}", nuser_id, level_and_exp_checks.1.unwrap_err().to_string());
        ctx.send(CreateReply::default()
            .content("Failed to check user's level! Please try again later or report this issue to <@908779319084589067>.")
            .ephemeral(true)
        ).await?;

        return Ok(());
    }

    let level: u64 = level_and_exp_checks.0.unwrap();
    let experience: u64 = level_and_exp_checks.1.unwrap();
    let experience_needed: u64 = exp_needed_to_next_level(level);
    let progressbar: String = generate_emoji_progressbar(experience, experience_needed, custom_data.config.leveling.progrees_bar_size, &custom_data.config.progressbar_emojis);
    let percentage: f64 = experience as f64/experience_needed as f64 *100.0;
    let percentage_text: String = format!("` {}% `", percentage.floor() as u64);

    ctx.send(CreateReply::default()
        .embed(CreateEmbed::default()
            .title(format!("{}'s Level Info", nuser.name))
            .description(format!("**Level:** {}\n**Experience:** {}/{}{} {}", level, experience, experience_needed, progressbar, percentage_text))
            .color(Color::from_rgb(255, 255, 255))
        )
    ).await?;

    Ok(())
}

#[derive(PartialEq)]
#[derive(poise::ChoiceParameter)]
pub enum LeaderboardType {
    Level,
    Balance,
}

/// Leaderboard
#[poise::command(slash_command, aliases("lb", "top"))]
pub async fn leaderboard(
    ctx: Context<'_>,
    #[description = "Leaderboard Type"] lbtype: LeaderboardType
) -> Result<(), Error> {
    let db_client = &ctx.data().db_client;
    if lbtype == LeaderboardType::Level {
        let lb: Result<Vec<(u64, u64, u64)>, async_sqlite::Error> = build_level_leaderboard_from_eco_db(db_client).await;

        let mut response = String::from("");
        for (index, (discord_id, level, experience)) in lb.unwrap().iter().enumerate() {
            let placement_emoji: &str;
            if index == 0 {
                placement_emoji = "<a:WINNER:1367093328864346122>";
            } else if index == 1 {
                placement_emoji = ":second_place:";
            } else if index == 2 {
                placement_emoji = ":third_place:";
            } else {
                placement_emoji = "";
            }

            let experience_needed: u64 = exp_needed_to_next_level(*level);
            let custom_data = &ctx.data();
            let progressbar: String = generate_emoji_progressbar(*experience, experience_needed, custom_data.config.leveling.progress_bar_in_leaderboard_size, &custom_data.config.progressbar_emojis);

            response.push_str(&format!("{} **{}.** <@{}> •\n<:LoopchanLevel:1368298876842279072> Level: {}\n<:LoopchanExp:1368298874803982479> Experience: {}/{}{}\n\n", placement_emoji, index + 1, discord_id, level, experience, experience_needed, progressbar));
        }

        response.push_str("-# Leaderboard is limited to 5 places.");
        let placement: Result<u8, async_sqlite::Error> = get_user_placement_in_level_leaderboard(db_client, ctx.author().id.get()).await;
        if placement.is_ok() {
            response.push_str(&format!("\n-# Your placement is #{}. <a:haphap:1367093618967318618>", placement.unwrap()));
        } else {
            response.push_str("\n-# Failed to fetch your placement.");
            error!("Failed to fetch {}'s placement: {}", ctx.author().name, placement.unwrap_err().to_string());
        }
        
        ctx.send(CreateReply::default()
            .embed(CreateEmbed::default()
                .title("<a:qtstar:1367089440073318501> Level Leaderboard")
                .description(response)
                .color(Color::from_rgb(255, 255, 255))
            )
        ).await?;

        return Ok(());
    }

    let lb: Result<Vec<(u64, u64)>, async_sqlite::Error> = build_balance_leaderboard_from_eco_db(db_client).await;

    let mut response = String::from("");

    for (index, (discord_id, balance)) in lb.unwrap().iter().enumerate() {
        if index == 0 {
            response.push_str(&format!("<a:WINNER:1367093328864346122> **1.** <@{}> •\n<:LoopchanCoin:1368311103238570025> Balance: {}\n\n", discord_id, balance));
            continue;
        }
        if index == 1 {
            response.push_str(&format!(":second_place: **2.** <@{}> •\n<:LoopchanCoin:1368311103238570025> Balance: {}\n\n", discord_id, balance,));
            continue;
        }
        if index == 2 {
            response.push_str(&format!(":third_place: **3.** <@{}> •\n<:LoopchanCoin:1368311103238570025> Balance: {}\n\n", discord_id, balance));
            continue;
        }
        response.push_str(&format!("**{}.** <@{}> •\n<:LoopchanCoin:1368311103238570025> Balance: {}\n\n", index + 1, discord_id, balance));
    }

    response.push_str("-# Leaderboard is limited to 5 places.");
    let placement: Result<u8, async_sqlite::Error> = get_user_placement_in_balance_leaderboard(db_client, ctx.author().id.get()).await;
    if placement.is_ok() {
        response.push_str(&format!("\n-# Your placement is #{}.", placement.unwrap()));
    } else {
        response.push_str("\n-# Failed to fetch your placement.");
        error!("Failed to fetch {}'s placement: {}", ctx.author().name, placement.unwrap_err().to_string());
    }
    
    ctx.send(CreateReply::default()
        .embed(CreateEmbed::default()
            .title("<a:qtstar:1367089440073318501> Balance Leaderboard")
            .description(response)
            .color(Color::from_rgb(255, 255, 255))
        )
    ).await?;

    Ok(())
}

/// A role shop
#[poise::command(slash_command, aliases("rs"))]
pub async fn roleshop(
    ctx: Context<'_>
) -> Result<(), Error> {
    let loopchans_config: &crate::LoopchanConfig = &ctx.data().config;
    let shop_items: &Vec<toml::Value> = &loopchans_config.economy.shop_items;

    let mut options_vec: Vec<CreateSelectMenuOption> = vec![];

    let mut response: String = String::new();
    let mut item_index: u8 = 0;
    for item in shop_items {
        item_index += 1;
        let item_unwrapped: &toml::map::Map<String, toml::Value> = item.as_table().unwrap();
        let item_prepared: RoleShopItem = RoleShopItem { // I hate this, what the actual fuck is this?? .unwrap().unwrap().unwrap().unwrap().unwrap().unwrap().unwrap().unwrap().unwrap().unwrap().unwrap().unwrap().unwrap().unwrap()
            id: item_unwrapped.get("id").unwrap().as_integer().unwrap() as u64,
            icon_id: item_unwrapped.get("icon_id").unwrap().as_integer().unwrap() as u64,
            icon_name: item_unwrapped.get("icon_name").unwrap().as_str().unwrap().to_string(),
            display_name: item_unwrapped.get("display_name").unwrap().as_str().unwrap().to_string(),
            description: item_unwrapped.get("description").unwrap().as_str().unwrap().to_string(),
            price: item_unwrapped.get("price").unwrap().as_integer().unwrap() as u32,
        }; // TODO: Make roles buyable and give user role with item_prepared ID

        let item_emoji: ReactionType = ReactionType::Custom {
            animated: false,
            id: item_prepared.icon_id.into(),
            name: Some(item_prepared.icon_name.clone())
        };

        options_vec.push(
            CreateSelectMenuOption::new(format!("{} • ${}", &item_prepared.display_name, &item_prepared.price), format!("roleshop.{}", item_prepared.id))
                .emoji(item_emoji)
                .description(&item_prepared.description)
        );

        response.push_str(&format!("<:{}:{}> **{}.** {} • *${}*\n*{}*\n\n",
            item_prepared.icon_name,
            item_prepared.icon_id,
            item_index,
            item_prepared.display_name,
            item_prepared.price,
            item_prepared.description,
        ));
    }

    let components: Vec<CreateActionRow> = vec![
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new("roleshopselector",
            CreateSelectMenuKind::String {
                options: options_vec
            }
        )
        )
    ];

    ctx.send(CreateReply::default()
        .embed(CreateEmbed::default()
            .description(format!("# Role Shop\n{}", response))
            .color(Color::from_rgb(255, 255, 255))
        )
        .components(components)
    ).await?;

    Ok(())
}

/// Work a parkourian job
#[poise::command(slash_command)] // TODO: Make this look pretty
pub async fn work(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let economy_config = &ctx.data().config.economy;
    let on_cooldown: i32;

    {
        let mut cooldown_tracker = ctx.command().cooldowns.lock().unwrap();

        let mut cooldown_durations: CooldownConfig = CooldownConfig::default();
        cooldown_durations.user = Some(Duration::from_secs(economy_config.work_cooldown*60));

        match cooldown_tracker.remaining_cooldown(ctx.cooldown_context(), &cooldown_durations) {
            Some(remaining) => {
                on_cooldown = remaining.as_secs() as i32;
            }
            None => {
                cooldown_tracker.start_cooldown(ctx.cooldown_context());
                on_cooldown = -1;
            },
        }
    };

    if on_cooldown != -1 {
        if on_cooldown > 60 {
            ctx.send(
                CreateReply::default()
                    .embed(
                        CreateEmbed::default()
                            .description(format!("You're currently too exhausted to work! Wait `{}` minutes.\n-# {} seconds", on_cooldown/60, on_cooldown))
                            .color(Color::from_rgb(255, 100, 100))
                    )
                    .ephemeral(true)
            ).await?;

            return Ok(());
        }

        ctx.send(
            CreateReply::default()
                .embed(
                    CreateEmbed::default()
                        .description(format!("You're currently too exhausted to work! Wait `{}` seconds.", on_cooldown))
                        .color(Color::from_rgb(255, 100, 100))
                )
                .ephemeral(true)
        ).await?;

        return Ok(());
    }
    
    if rand::rng().random_bool((1.0-economy_config.work_fail_chance).into()) == false { // Failed work
        let random_phrase = rand::rng().random_range(0..economy_config.failed_work_phrases.len());
        ctx.send(
            CreateReply::default()
                .embed(
                    CreateEmbed::default()
                        .description(economy_config.failed_work_phrases[random_phrase].as_str().unwrap())
                        .color(Color::from_rgb(255, 100, 100))
                )
        ).await?;

        return Ok(());
    }

    let add_to_balance: u64 = rand::rng().random_range(economy_config.work_payment[0].as_integer().unwrap()..economy_config.work_payment[1].as_integer().unwrap()).try_into().unwrap();
    increment_user_balance_in_eco_db(&ctx.data().db_client, ctx.author().id.get(), add_to_balance).await?; // TODO: Handle error

    let random_phrase_num = rand::rng().random_range(0..economy_config.work_phrases.len());
    let random_phrase = economy_config.work_phrases[random_phrase_num].as_str().unwrap();
    ctx.send(
        CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .description(random_phrase.replace("{}", &add_to_balance.to_string()))
                    .color(Color::from_rgb(100, 255, 100))
            )
    ).await?;

    Ok(())
}