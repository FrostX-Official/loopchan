use dotenv::dotenv;

use poise::serenity_prelude as serenity;
use roboat;

mod commands;

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

struct Data {
    roblox_client: roboat::Client,
    db_client: async_sqlite::Client,
    guild_id: u64,
    staff_role_id: u64,
    qa_role_id: u64
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token: String = std::env::var("LOOPLINK_DISCORD_TOKEN").expect("missing LOOPLINK_DISCORD_TOKEN");

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

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::hello::hello(),
                commands::rbx::fetchdata(),
                commands::qa::qa(),
                commands::qa::status(),
            ],
            pre_command: |ctx| {
                let author: &serenity::model::prelude::User = ctx.author();
                let author_id: u64 = author.id.get();

                let custom_data = ctx.data();

                let guild_id: u64 = custom_data.guild_id;
                let staff_role_id: u64 = custom_data.staff_role_id;
                let qa_role_id: u64 = custom_data.qa_role_id;
                

                Box::pin(async move {
                    println!("@{} ({}) executing command: \"{}\"", author.name, author.id, ctx.command().name);

                    let is_staff: bool = author.has_role(ctx, guild_id, staff_role_id).await.unwrap_or(false);
                    let is_qa: bool = author.has_role(ctx, guild_id, qa_role_id).await.unwrap_or(false);

                    let _ = &custom_data.db_client.conn(move |conn| {
                        conn.execute(
                            "INSERT INTO users (discord_id) VALUES (?1, ?2, ?3, ?4) ON CONFLICT DO UPDATE SET
                            discord_id=excluded.discord_id
                            roblox_id=excluded.discord_id
                            staff=excluded.staff
                            qa=excluded.qa",
                            (author_id, 0, is_staff, is_qa) // TODO: Roblox Linking in rbx.rs
                        )
                    }).await;
                })
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            println!("Ready!");
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    roblox_client: roboat::ClientBuilder::new().build(),
                    db_client: ClientBuilder::new()
                        .path("users.db")
                        .open()
                        .await.expect("Failed connecting to sqlite"),
                    guild_id: std::env::var("PTL_GUILD_ID").expect("missing PTL_GUILD_ID").parse().unwrap(),
                    staff_role_id: std::env::var("STAFF_ROLE_ID").expect("missing STAFF_ROLE_ID").parse().unwrap(),
                    qa_role_id: std::env::var("QA_ROLE_ID").expect("missing QA_ROLE_ID").parse().unwrap()
                })
            })
        })
        
        .build();

    let mut client = serenity::ClientBuilder::new(token, serenity::GatewayIntents::non_privileged())
        .framework(framework)
        .await
        .expect("Err creating client");

    client.start().await.unwrap();
}