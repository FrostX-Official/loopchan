use dotenv::dotenv;

// Logging
use std::io::Write;
use env_logger::Builder;
use chrono::Local;
use log::LevelFilter;

use poise::serenity_prelude as serenity;
use roboat;

mod commands;
mod utils;

use async_sqlite::ClientBuilder;

// #[derive(Default)]
// #[derive(Debug)]
// pub struct LoopchanUser {
//     discord_id: u64,
//     roblox_id: u64,
//     staff: bool,
//     qa: bool,
//     //prime: bool,
//     //alpha: bool,
//     //beta: bool,
// }

// Data, which is stored and accessible in all command invocations
#[allow(dead_code)]
struct Data {
    roblox_client: roboat::Client, // Used for interactions with Roblox API
    db_client: async_sqlite::Client, // Used for interactions with Loopchan's Database
    // Misc Variables
    guild_id: u64,
    // these will be useful later
    staff_role_id: u64,
    qa_role_id: u64
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Loopchan's Game Status
use once_cell::sync::Lazy;
static PTL_PAID_TESTING_PRESENCE: Lazy<serenity::ActivityData> = Lazy::new(|| serenity::ActivityData {
    name: "PTL Paid Testing".to_string(),
    kind: serenity::ActivityType::Playing,
    state: None,
    url: None,
});

#[tokio::main]
async fn main() {
    // .env
    dotenv().ok();

    // Logger
    Builder::new()
        .format(|buf: &mut env_logger::fmt::Formatter, record| {
            writeln!(buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            ) 
        })
        .filter(None, LevelFilter::Info)
        .init();

    // Loopchan's Database
    let sqlite_client: async_sqlite::Client = ClientBuilder::new()
        .path("users.db")
        .open()
        .await.expect("Failed connecting to sqlite");

    utils::db::prepare_users_db(&sqlite_client).await;

    // Loopchan's Poise Framework
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::debug::debug(),
                commands::rbx::fetchdata(),
                commands::rbx::verify(),
                commands::qa::qa()
            ],
            // TODO: Work on giving every command cooldown through command check
            // command_check: Some(|ctx| {
            //     Box::pin(async move {
            //         let mut cooldown_tracker = ctx.command().cooldowns.lock().unwrap();

            //         let mut cooldown_durations: poise::CooldownConfig = poise::CooldownConfig::default();

            //         let remaining_cooldown = cooldown_tracker.remaining_cooldown(ctx.cooldown_context(), &cooldown_durations);
            //         cooldown_durations.user = Some(std::time::Duration::from_secs(5));
    
            //         match remaining_cooldown {
            //             Some(remaining) => {
            //                 return Err(format!("Please wait {} seconds", remaining.as_secs()).into())
            //             }
            //             None => {
            //                 cooldown_tracker.start_cooldown(ctx.cooldown_context())
            //             },
            //         }
            //         Ok(true)
            //     })
            // }),
            pre_command: |ctx| {
                let author: &serenity::model::prelude::User = ctx.author();
                let author_id: u64 = author.id.get();

                let custom_data: &Data = ctx.data();

                Box::pin(async move {
                    log::info!("@{} ({}) executing command: \"{}\"", author.name, author.id, ctx.command().name);

                    utils::db::create_user_in_db(&custom_data.db_client, author_id, 0).await.expect("Failed to create user in database in pre-command hook!");
                })
            },
            //manual_cooldowns: true,
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            ctx.set_activity(Some(PTL_PAID_TESTING_PRESENCE.clone()));
            ctx.dnd();

            log::info!("Ready!");

            Box::pin(async move {
                let ptl_guild_id: serenity::model::prelude::GuildId = std::env::var("PTL_GUILD_ID").expect("missing PTL_GUILD_ID").parse().unwrap();
                // Register commands
                //poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                poise::builtins::register_in_guild(&ctx.http, &framework.options().commands, ptl_guild_id).await?;
                // Create global data for commands and hooks
                Ok(Data {
                    roblox_client: roboat::ClientBuilder::new().build(),
                    db_client: sqlite_client,
                    guild_id: ptl_guild_id.into(),
                    staff_role_id: std::env::var("STAFF_ROLE_ID").expect("missing STAFF_ROLE_ID").parse().unwrap(),
                    qa_role_id: std::env::var("QA_ROLE_ID").expect("missing QA_ROLE_ID").parse().unwrap()
                })
            })
        })
        .build();

    // Loopchan Start
    let token: String = std::env::var("LOOPCHAN_DISCORD_TOKEN").expect("missing LOOPCHAN_DISCORD_TOKEN");
    let mut client: serenity::Client = serenity::ClientBuilder::new(token, serenity::GatewayIntents::all())
        .framework(framework)
        .await
        .expect("Err creating client");

    client.start_autosharded().await.unwrap();
}