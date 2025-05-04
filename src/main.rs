use dotenv::dotenv;

use ::serenity::all::{ChannelId, Color, ComponentInteraction, CreateAllowedMentions, CreateEmbed, CreateMessage, EditMessage};

use poise::CreateReply;
use poise::FrameworkError;
use poise::serenity_prelude as serenity;

use roboat;

use ::serenity::prelude::TypeMapKey;

use utils::db::{create_db, prepare_users_db, prepare_eco_db, create_user_in_users_db, create_user_in_eco_db};

use tokio::sync::Mutex;
use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;

// Config
use serde::Deserialize;
use toml;

#[derive(Deserialize)]
pub struct LoopchanConfig {
    guild: u64,
    owner: u64,
    global_cooldown: u64,
    database_path: Option<String>,
    welcomecard: WelcomecardConfig,
    roles: LoopchansRoles,
    channels: LoopchansChannels
}

impl TypeMapKey for LoopchanConfig {
    type Value = LoopchanConfig;
}

#[derive(Deserialize)]
pub struct WelcomecardConfig {
    enabled: bool,
    channel: Option<u64>,
    react: Option<bool>,
    react_id: Option<u64>,
    react_name: Option<String>,
    react_animated: Option<bool>,
}

#[derive(Deserialize)]
pub struct LoopchansRoles {
    qa: u64,
    staff: u64,
    member: u64,
}

#[derive(Deserialize)]
pub struct LoopchansChannels {
    qa_forms: u64,
    unverified_chat: u64
}

// Logging
use chrono::Local;
use tracing::level_filters::LevelFilter;
use tracing::{info, warn, error};
use tracing_appender::rolling;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;
use tracing_subscriber::fmt::{time, format};

struct LocalTime;

impl time::FormatTime for LocalTime {
    fn format_time(&self, w: &mut format::Writer<'_>) -> std::fmt::Result {
        let now = Local::now();
        write!(w, "{}", now.format("%Y-%m-%d %H:%M:%S%.3f"))
    }
}

// Other modules ("cogs")
mod handlers;
mod commands;
mod utils;

// Data, which is stored and accessible in all command invocations
#[allow(dead_code)]
struct Data {
    roblox_client: roboat::Client, // Used for interactions with Roblox API
    db_client: async_sqlite::Client, // Used for interactions with Loopchan's Database
    exp_cooldowns: Mutex<HashMap<u64, Instant>>, // Used to cooldown economics exp add
    config: LoopchanConfig
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

// Error Handler
async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        crate::FrameworkError::Setup { error, .. } => {
            eprintln!("Error in user data setup: {}", error);
        }
        crate::FrameworkError::EventHandler { error, event, .. } => error!(
            "User event event handler encountered an error on {} event: {}",
            event.snake_case_name(),
            error
        ),
        crate::FrameworkError::Command { ctx, error , .. } => {
            let error = error.to_string();
            eprintln!("An error occured in a command: {}", error);

            let mentions = CreateAllowedMentions::new()
                .everyone(false)
                .all_roles(false)
                .all_users(false);

            ctx.send(
                CreateReply::default()
                    .content(error)
                    .allowed_mentions(mentions),
            )
            .await.expect("Failed to send error message");
        }
        crate::FrameworkError::SubcommandRequired { ctx } => {
            let subcommands = ctx
                .command()
                .subcommands
                .iter()
                .map(|s| &*s.name)
                .collect::<Vec<_>>();
            let response = format!(
                "You must specify one of the following subcommands: {}",
                subcommands.join(", ")
            );
            ctx.send(CreateReply::default().content(response).ephemeral(true))
                .await.expect("Failed to send error message");
        }
        crate::FrameworkError::CommandPanic { ctx, payload: _ , .. } => {
            // Not showing the payload to the user because it may contain sensitive info
            let embed = serenity::CreateEmbed::default()
                .title("Internal error")
                .color((255, 0, 0))
                .description("An unexpected internal error has occurred.");

            ctx.send(CreateReply::default().embed(embed).ephemeral(true))
                .await.expect("Failed to send error message");
        }
        crate::FrameworkError::ArgumentParse { ctx, input, error, .. } => {
            // If we caught an argument parse error, give a helpful error message with the
            // command explanation if available
            let usage = match &ctx.command().help_text {
                Some(help_text) => &**help_text,
                None => "Please check the help menu for usage information",
            };
            let response = if let Some(input) = input {
                format!(
                    "**Cannot parse `{}` as argument: {}**\n{}",
                    input, error, usage
                )
            } else {
                format!("**{}**\n{}", error, usage)
            };

            let mentions = CreateAllowedMentions::new()
                .everyone(false)
                .all_roles(false)
                .all_users(false);

            ctx.send(
                CreateReply::default()
                    .content(response)
                    .allowed_mentions(mentions),
            )
            .await.expect("Failed to send error message");
        }
        crate::FrameworkError::CommandStructureMismatch { ctx, description, .. } => {
            error!(
                "Error: failed to deserialize interaction arguments for `/{}`: {}",
                ctx.command.name,
                description,
            );
        }
        crate::FrameworkError::CommandCheckFailed { ctx, error, .. } => {
            error!(
                "A command check failed in command {} for user {}: {:?}",
                ctx.command().name,
                ctx.author().name,
                error,
            );
        }
        crate::FrameworkError::CooldownHit {
            remaining_cooldown,
            ctx,
            ..
        } => {
            let msg = format!(
                "You're too fast!~ Please wait `{}` seconds before retrying!!",
                remaining_cooldown.as_secs()
            );
            ctx.send(CreateReply::default().content(msg).ephemeral(true))
                .await.expect("Failed to send error message");
        }
        crate::FrameworkError::MissingBotPermissions {
            missing_permissions,
            ctx,
            ..
        } => {
            let msg = format!(
                "Command cannot be executed because the bot is lacking permissions: {}",
                missing_permissions,
            );
            ctx.send(CreateReply::default().content(msg).ephemeral(true))
                .await.expect("Failed to send error message");
        }
        crate::FrameworkError::MissingUserPermissions {
            missing_permissions,
            ctx,
            ..
        } => {
            let response = if let Some(missing_permissions) = missing_permissions {
                format!(
                    "You're lacking permissions for `{}{}`: {}",
                    ctx.prefix(),
                    ctx.command().name,
                    missing_permissions,
                )
            } else {
                format!(
                    "You may be lacking permissions for `{}{}`. Not executing for safety",
                    ctx.prefix(),
                    ctx.command().name,
                )
            };
            ctx.send(CreateReply::default().content(response).ephemeral(true))
                .await.expect("Failed to send error message");
        }
        crate::FrameworkError::NotAnOwner { ctx, .. } => {
            let response = "Only bot owners can call this command";
            ctx.send(CreateReply::default().content(response).ephemeral(true))
                .await.expect("Failed to send error message");
        }
        crate::FrameworkError::GuildOnly { ctx, .. } => {
            let response = "You cannot run this command in DMs.";
            ctx.send(CreateReply::default().content(response).ephemeral(true))
                .await.expect("Failed to send error message");
        }
        crate::FrameworkError::DmOnly { ctx, .. } => {
            let response = "You cannot run this command outside DMs.";
            ctx.send(CreateReply::default().content(response).ephemeral(true))
                .await.expect("Failed to send error message");
        }
        crate::FrameworkError::NsfwOnly { ctx, .. } => {
            let response = "You cannot run this command outside NSFW channels.";
            ctx.send(CreateReply::default().content(response).ephemeral(true))
                .await.expect("Failed to send error message");
        }
        crate::FrameworkError::DynamicPrefix { error, msg, .. } => {
            error!(
                "Dynamic prefix failed for message {:?}: {}",
                msg.content,
                error
            );
        }
        crate::FrameworkError::UnknownCommand {
            msg_content,
            prefix,
            ..
        } => {
            warn!(
                "Recognized prefix `{}`, but didn't recognize command name in `{}`",
                prefix,
                msg_content,
            );
        }
        crate::FrameworkError::UnknownInteraction { interaction, .. } => {
            warn!("received unknown interaction \"{}\"", interaction.data.name);
        }
        crate::FrameworkError::__NonExhaustive(unreachable) => match unreachable {},
    }
}


async fn handle_message_component(
    ctx: &serenity::Context,
    _event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
    component: &ComponentInteraction
) -> Result<(), Error> {
    let loopchans_config = &data.config;
    if component.data.custom_id == "qa.invitation.accept" {
        let ptl_channels: std::collections::HashMap<ChannelId, serenity::model::prelude::GuildChannel> = ctx.cache.guild(loopchans_config.guild).unwrap().channels.clone();
        let qa_forms_channel = ptl_channels.get(&loopchans_config.channels.qa_forms.into());
        if qa_forms_channel.is_none() {
            warn!("Failed to get QA Forms Channel while user was accepting QA invitation!");
            return Ok(());
        }

        qa_forms_channel.unwrap().send_message(ctx, CreateMessage::default()
            .embed(
                CreateEmbed::default()
                    .title("QA Team Invitation")
                    .description(format!("@{} (<@{}>) have accepted QA Team Invitation!", component.user.name, component.user.id))
                    .color(Color::from_rgb(100, 255, 100))
            )
        ).await?;

        info!("@{} ({}) have accepted QA Team invitation!", component.user.name, component.user.id);
        component.message.clone().edit(ctx, 
            EditMessage::default()
            .embed(
                CreateEmbed::default()
                    .title("QA Team Invitation")
                    .description(
                        "QA Form reviewers have been notified about your application."
                    )
                    .color(Color::from_rgb(255, 255, 255))
            ).components(vec![])
        ).await?;

        component.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge).await?;
    } else if component.data.custom_id == "qa.invitation.deny" {
        let ptl_channels: std::collections::HashMap<ChannelId, serenity::model::prelude::GuildChannel> = ctx.cache.guild(loopchans_config.guild).unwrap().channels.clone();
        let qa_forms_channel = ptl_channels.get(&loopchans_config.channels.qa_forms.into());
        if qa_forms_channel.is_none() {
            warn!("Failed to get QA Forms Channel while user was accepting QA invitation!");
            return Ok(());
        }

        qa_forms_channel.unwrap().send_message(ctx, CreateMessage::default()
            .embed(
                CreateEmbed::default()
                    .title("QA Team Invitation")
                    .description(format!("@{} (<@{}>) have declined QA Team Invitation!", component.user.name, component.user.id))
                    .color(Color::from_rgb(255, 100, 100))
            )
        ).await?;

        info!("@{} ({}) have declined QA Team invitation!", component.user.name, component.user.id);
        component.message.clone().edit(ctx, 
            EditMessage::default()
            .embed(
                CreateEmbed::default()
                    .title("QA Team Invitation")
                    .description(
                        "You have declined the invitation."
                    )
                    .color(Color::from_rgb(255, 100, 100))
            ).components(vec![])
        ).await?;
        component.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge).await?;
    }

    Ok(())
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => { // Print bot's username on startup
            warn!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::InteractionCreate { interaction } => { // Different interactions handling
            // Message Component
            let is_component: Option<ComponentInteraction> = interaction.clone().into_message_component();
            if !is_component.is_none() { return handle_message_component(ctx, event, framework, data, &is_component.unwrap()).await; }
        }
        serenity::FullEvent::Message { new_message } => {
            if new_message.author.bot { return Ok(()); }
            if new_message.content.len() < 2 { return Ok(()); }
            let userid: u64 = new_message.author.id.get();
            let cooldown_duration: Duration = Duration::from_secs(10);
            let mut cooldowns = data.exp_cooldowns.lock().await;
            let last_exp_time = cooldowns.entry(userid).or_insert(Instant::now() - cooldown_duration);

            if last_exp_time.elapsed() >= cooldown_duration {
                let exp_amount: u64 = new_message.content.len().min(25) as u64;
                commands::eco::give_user_eco_exp(data, &new_message.author, exp_amount).await;
                *last_exp_time = Instant::now();
            } else {
                info!("Tried to give {} exp after message, but it's on cooldown", userid)
            }
        }
        serenity::FullEvent::GuildMemberAddition { new_member } => { // WELCOMECARD // WELCOME MESSAGE
            handlers::events::welcomecard::welcomecard(ctx, new_member, data).await?;
        }
        _ => {}
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    // .env
    dotenv().ok();

    // Logger
    let timern = Local::now().format("%Y-%m-%d %H-%M-%S");
    let file_appender = rolling::never("logs", timern.to_string()+".log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let log_file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(false)
        .with_span_events(FmtSpan::CLOSE)
        .event_format(tracing_subscriber::fmt::format().with_timer(LocalTime).compact())
        .with_filter(LevelFilter::DEBUG);

    let terminal_layer = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_target(false)
        .with_span_events(FmtSpan::CLOSE)
        .event_format(tracing_subscriber::fmt::format().with_timer(LocalTime).compact())
        .with_filter(LevelFilter::WARN);

    tracing_subscriber::registry()
        .with(log_file_layer)
        .with(terminal_layer)
        .init();

    let toml_string = tokio::fs::read_to_string("Config.toml").await;
    if toml_string.is_err() {
        error!("Failed to read your config.toml file: {}", toml_string.unwrap_err().to_string());
        return;
    }
    let loopchans_config: LoopchanConfig = toml::from_str(&toml_string.unwrap()).unwrap();

    // Loopchan's Database
    let sqlite_client: async_sqlite::Client = create_db(loopchans_config.database_path).await.expect("Failed connecting to users database");
    prepare_users_db(&sqlite_client).await;
    prepare_eco_db(&sqlite_client).await;

    // Loopchan's Poise Framework
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::debug::debug(),
                commands::rbx::fetchdata(),
                commands::rbx::verify(),
                commands::qa::qa(),
                commands::eco::eco(),
            ],
            command_check: Some(|ctx| {
                Box::pin(async move {
                    let loopchans_config = &ctx.data().config;
                    let mut cooldown_durations = poise::CooldownConfig::default();
                    cooldown_durations.user = Some(std::time::Duration::from_secs(loopchans_config.global_cooldown));

                    let cc: poise::CooldownContext = poise::CooldownContext {
                        user_id: ctx.author().id,
                        channel_id: ctx.channel_id(),
                        guild_id: ctx.guild_id()
                    };

                    let remaining_cooldown = {
                        let cooldown_tracker = ctx.command().cooldowns.lock().unwrap();
                        cooldown_tracker.remaining_cooldown(cc, &cooldown_durations)
                    };

                    match remaining_cooldown {
                        Some(remaining) => {
                            let remaining_precise: f64 = (remaining.as_millis() as f64)/1000.0;
                            let error_msg = format!("You're too fast!~ Please wait `{}` seconds before retrying!!", remaining_precise);
                            //warn!(error_msg);

                            ctx.send(poise::CreateReply::default()
                                .content(error_msg)
                                .ephemeral(true)
                            ).await?;
                            Err(format!("Cooldown {} seconds", remaining_precise).into())
                        }
                        None => {
                            // Moved to post_command hook
                            //let mut cooldown_tracker = ctx.command().cooldowns.lock().unwrap();
                            //cooldown_tracker.start_cooldown(cc);
                            Ok(true)
                        },
                    }
                })
            }),
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            on_error: |error| Box::pin(on_error(error)),
            pre_command: |ctx| {
                let author: &serenity::model::prelude::User = ctx.author();
                let author_id: u64 = author.id.get();

                let custom_data: &Data = ctx.data();

                Box::pin(async move {
                    info!("@{} ({}) executing command: \"{}\"", author.name, author.id, ctx.command().name);

                    create_user_in_users_db(&custom_data.db_client, author_id, 0).await.expect("Failed to create user in users database in pre-command hook!");
                    create_user_in_eco_db(&custom_data.db_client, author_id).await.expect("Failed to create user in economics database in pre-command hook!");
                })
            },
            post_command: |ctx| {
                let author = ctx.author();
                let cc: poise::CooldownContext = poise::CooldownContext {
                    user_id: author.id,
                    channel_id: ctx.channel_id(),
                    guild_id: ctx.guild_id()
                };
                let mut cooldown_tracker = ctx.command().cooldowns.lock().unwrap();
                cooldown_tracker.start_cooldown(cc);

                Box::pin(async move {
                    info!("@{} ({}) executed command: \"{}\"", author.name, author.id, ctx.command().name);
                })
            },
            //manual_cooldowns: true,
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                let toml_string = tokio::fs::read_to_string("Config.toml").await;
                if toml_string.is_err() {
                    let error_str = format!("Failed to read your config.toml file: {}", toml_string.unwrap_err().to_string());
                    error!(error_str);
                    return Err(error_str.into());
                }

                let loopchans_config: LoopchanConfig = toml::from_str(&toml_string.unwrap()).unwrap();

                ctx.set_activity(Some(PTL_PAID_TESTING_PRESENCE.clone()));
                ctx.dnd();

                info!("Ready!");

                let ptl_guild_id: serenity::model::prelude::GuildId = loopchans_config.guild.into();
                // Register commands
                //poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                poise::builtins::register_in_guild(&ctx.http, &framework.options().commands, ptl_guild_id).await?;
                // Create global data for commands and hooks
                Ok(Data {
                    roblox_client: roboat::ClientBuilder::new().build(),
                    db_client: sqlite_client,
                    exp_cooldowns: Mutex::new(HashMap::new()),
                    config: loopchans_config
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