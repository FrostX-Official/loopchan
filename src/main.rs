use dotenv::dotenv;

// Logging
use std::io::Write;
use env_logger::Builder;
use chrono::Local;
use log::LevelFilter;

use once_cell::sync::Lazy;
use poise::serenity_prelude as serenity;
use roboat::{self};

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
struct Data {
    roblox_client: roboat::Client, // Used for interactions with Roblox API
    db_client: async_sqlite::Client, // Used for interactions with Loopchan's Database
    // Misc Variables
    guild_id: u64,
    staff_role_id: u64,
    qa_role_id: u64
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Loopchan's Game Status
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

    sqlite_client.conn(|conn: &async_sqlite::rusqlite::Connection| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                discord_id INTEGER PRIMARY KEY,
                roblox_id INTEGER,
                staff BOOLEAN,
                qa BOOLEAN
            )",
            []
        )
    }).await.unwrap();

    // Loopchan's Poise Framework
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::debug::debug(),
                commands::rbx::fetchdata(),
                commands::qa::qa()
            ],
            pre_command: |ctx| {
                let author: &serenity::model::prelude::User = ctx.author();
                let author_id: u64 = author.id.get();

                let custom_data: &Data = ctx.data();

                let guild_id: u64 = custom_data.guild_id;
                let staff_role_id: u64 = custom_data.staff_role_id;
                let qa_role_id: u64 = custom_data.qa_role_id;

                Box::pin(async move {
                    log::info!("@{} ({}) executing command: \"{}\"", author.name, author.id, ctx.command().name);

                    let is_staff: bool = author.has_role(ctx, guild_id, staff_role_id).await.unwrap_or(false);
                    let is_qa: bool = author.has_role(ctx, guild_id, qa_role_id).await.unwrap_or(false);

                    // TODO: Roblox Linking in rbx.rs
                    let _ = utils::db::create_user_in_db(&custom_data.db_client, author_id, 0, is_staff, is_qa).await;
                })
            },
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