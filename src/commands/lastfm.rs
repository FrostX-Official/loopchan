use std::any::Any;
use std::time::Duration;

use serenity::all::{ButtonStyle, Color, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter};
use tracing::{error, warn};
use crate::utils::database::lastfm::{save_lastfm_session_data, get_lastfm_session_data};
use crate::{Context, Error};

use lastfm_rust::Lastfm;
use lastfm_rust::APIResponse;

/// Last.fm API Commands
#[poise::command(slash_command, subcommands("authorize", "currentlyplaying"), subcommand_required)]
pub async fn lastfm(_ctx: Context<'_>) -> Result<(), Error> { Ok(()) }

// Last.fm Token Generator
pub async fn generate_token(lastfm: &Lastfm) -> Result<String, lastfm_rust::Error> {
    let response: Result<APIResponse<lastfm_rust::AuthGetTokenResponse>, lastfm_rust::Error> = lastfm.auth().get_token().send().await;

    if response.is_err() {
        return Err(response.unwrap_err());
    }

    let token = match response.unwrap() {
        APIResponse::Success(value) => value.token,
        APIResponse::Error(err) => {
            return Err(err.into());
        }
    };

    Ok(token)
}

/// Authorize loopchan into your Last.fm account
#[poise::command(slash_command)]
pub async fn authorize(ctx: Context<'_>) -> Result<(), Error> {
    let author: &serenity::model::prelude::User = ctx.author();
    let custom_data: &crate::Data = ctx.data();

    let dont_check_session: bool;

    let get_session_data: Result<(String, String), async_sqlite::Error> = get_lastfm_session_data(&custom_data.db_client, author.id.get()).await;
    if get_session_data.is_err() {
        let err_unwrapped = get_session_data.unwrap_err();
        if err_unwrapped.type_id() == async_sqlite::Error::Rusqlite(async_sqlite::rusqlite::Error::QueryReturnedNoRows).type_id() {
            dont_check_session = true;
        } else {
            error!("Failed to get user's ({}) Last.fm session data: {}", author.id.get(), err_unwrapped.to_string());
            ctx.send(poise::CreateReply::default()
                .embed(
                    CreateEmbed::default()
                        .description("Failed to check if you already authorized! Please try again later, if the issue persists contact <@908779319084589067>")
                        .color(Color::from_rgb(255, 100, 100))
                )
                .ephemeral(true)
            ).await?;
            return Ok(());
        }
    } else {
        dont_check_session = false;
    }

    if !dont_check_session {
        let reply: poise::ReplyHandle<'_> = ctx.send(poise::CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .description("You already have a session key active.\n**Are you sure** you want to regenerate it?")
                    .color(Color::from_rgb(255, 100, 100))
            )
            .components(vec![ // nesting hell
                CreateActionRow::Buttons(vec![
                    CreateButton::new("confirm")
                        .label("Yes!")
                        .style(ButtonStyle::Success),
                    CreateButton::new("cancel")
                        .label("Nuh uh")
                        .style(ButtonStyle::Danger),
            ])])
            .ephemeral(true)
        ).await?;

        let interaction_not_timed_out: Option<serenity::model::prelude::ComponentInteraction> = reply
            .message()
            .await?
            .await_component_interaction(ctx)
            .author_id(author.id)
            .timeout(Duration::new(60, 0))
            .await;
        
        reply
            .edit(
                ctx,
                poise::CreateReply::default()
                    .components(vec![])
                    .content("Processing..."),
            )
            .await?;

        if interaction_not_timed_out.is_none() {
            reply
                .edit(
                    ctx,
                    poise::CreateReply::default()
                        .content("Timed out."),
                )
                .await?;

            return Ok(());
        }

        let interaction = interaction_not_timed_out.unwrap();
        
        if interaction.data.custom_id == "cancel" {
            reply
                .edit(
                    ctx,
                    poise::CreateReply::default()
                        .content("Cancelled."),
                )
                .await?;

            return Ok(());
        }
    }

    let lastfm: &Lastfm = &custom_data.lastfm_client;
    let token_response: Result<String, lastfm_rust::Error> = generate_token(lastfm).await;

    if token_response.is_err() {
        error!("Failed to generate token for user ({}): {}", author.id.get(), token_response.unwrap_err().to_string());
        ctx.send(poise::CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .description("Failed to generate a token for you! Please try again later, if the issue persists contact <@908779319084589067>")
                    .color(Color::from_rgb(255, 100, 100))
            )
            .ephemeral(true)
        ).await?;
        return Ok(());
    }

    let token: String = token_response.unwrap();

    let auth_url: String = format!(
        "https://www.last.fm/api/auth?api_key={}&token={}",
        lastfm.get_api_key(),
        token.replace("\"", "")
    );

    let auth_reply: Result<poise::ReplyHandle<'_>, serenity::Error> = ctx.send(poise::CreateReply::default()
        .embed(
            CreateEmbed::default()
                .description("Press `Done` button after confirmed `Connect` on Last.fm website.\n*(you have 5 minutes or less ||(i forgor)||)*")
                .color(Color::from_rgb(255, 255, 255))
        )
        .components(vec![ // nesting hell
            CreateActionRow::Buttons(vec![
                CreateButton::new_link(auth_url)
                    .label("Connect")
                    .style(ButtonStyle::Primary),
                CreateButton::new(format!("lastfm.authorized.{}.{}", author.id.get(), token))
                    .label("Done")
                    .style(ButtonStyle::Success),
                CreateButton::new("lastfm.cancel_auth")
                    .label("Cancel")
                    .style(ButtonStyle::Danger)
        ])])
        .ephemeral(true)
    ).await;

    if auth_reply.is_err() {
        error!("Failed to send reply to user ({}) with auth URL: {}", author.id.get(), auth_reply.err().unwrap().to_string()); // idk why unwrap_err() doesn't work here T-T
        return Ok(());
    }

    let reply: poise::ReplyHandle<'_> = auth_reply.unwrap();
    let interaction_not_timed_out = reply
        .message()
        .await?
        .await_component_interaction(ctx)
        .author_id(author.id)
        .timeout(Duration::new(300, 0))
        .await;

    if interaction_not_timed_out.is_none() {
        reply
            .edit(
                ctx,
                poise::CreateReply::default()
                    .components(vec![])
                    .content("Timed out.")
            )
            .await?;
        return Ok(());
    } else {
        reply
            .edit(
                ctx,
                poise::CreateReply::default()
                    .components(vec![])
                    .content("Processing... Please wait."),
            )
            .await?;
    }
    
    let interaction: serenity::model::prelude::ComponentInteraction = interaction_not_timed_out.unwrap();
    if interaction.data.custom_id == "lastfm.cancel_auth" {
        reply
            .edit(
                ctx,
                poise::CreateReply::default()
                    .content("Cancelled authorization.")
            )
            .await?;
        return Ok(());
    }

    let get_session_result = lastfm.auth().get_session().token(&token).send().await;
    if get_session_result.is_err() {
        error!("Failed to claim user's ({}) session key: {}", author.id.get(), get_session_result.unwrap_err().to_string()); 
        reply
            .edit(
                ctx,
                poise::CreateReply::default()
                    .content("Failed to claim your session key. Please try again later, if the issue persists contact <@908779319084589067>")
            )
            .await?;
        return Ok(());
    }

    let session_data = &get_session_result.unwrap()["session"];
    let session_key = &session_data["key"];
    let session_username = &session_data["name"];
    warn!("{}'s last.fm session key: {} / username: {}", author.name, session_key, session_username);
    let successful_save: Result<usize, async_sqlite::Error> = save_lastfm_session_data(&custom_data.db_client, author.id.get(), session_key.as_str().unwrap().to_owned(), session_username.as_str().unwrap().to_owned()).await;

    if successful_save.is_err() {
        error!("Failed to save user's ({}) session key: {}", author.id.get(), successful_save.unwrap_err().to_string()); 
        reply
            .edit(
                ctx,
                poise::CreateReply::default()
                    .content("Failed to save your session key. Please try again later, if the issue persists contact <@908779319084589067>")
            )
            .await?;
        return Ok(());
    }

    let userinfo_response = lastfm.user().get_info().user(session_username.as_str().unwrap()).send().await.unwrap();
    let userinfo: Option<serde_json::Value>;

    match userinfo_response {
        APIResponse::Success(real_userinfo) => {
            userinfo = Some(real_userinfo);
        },
        APIResponse::Error(_) => { userinfo = None; }, // TODO: I am not sure if it can return error, but if it can do that add handle to that later
    }

    if userinfo.is_some() {
        let user = &userinfo.unwrap()["user"];
        reply
            .edit(
                ctx,
                poise::CreateReply::default()
                    .content("")
                    .embed(
                        CreateEmbed::default()
                            .title("Authorized!")
                            .thumbnail(user["image"][3]["#text"].as_str().unwrap())
                            .description(format!("<@{}> successfully authorized as **{}**!\nCurrently you have **{}** scrobbles,\nwith **{}** of them being unique :3",
                                author.id.get(),
                                user["name"].as_str().unwrap(),
                                user["playcount"].as_str().unwrap(),
                                user["track_count"].as_str().unwrap()
                            ))
                            .color(Color::from_rgb(100, 255, 100))
                    )
                    .ephemeral(false)
            )
            .await?;
    } else {
        reply
            .edit(
                ctx,
                poise::CreateReply::default()
                    .content("")
                    .embed(
                        CreateEmbed::default()
                            .title("Authorized!")
                            .description(format!("<@{}> successfully authorized as {}!", author.id.get(), session_username))
                            .footer(CreateEmbedFooter::new("Also failed to fetch more data about you.. T-T"))
                            .color(Color::from_rgb(100, 255, 100))
                    )
                    .ephemeral(false)
            )
            .await?;
    }

    Ok(())
}

/// Get your currently playing track.
#[poise::command(slash_command)]
pub async fn currentlyplaying(ctx: Context<'_>) -> Result<(), Error> {
    let custom_data: &crate::Data = ctx.data();
    let db_client = &custom_data.db_client;
    let author = ctx.author();

    let get_session_data: Result<(String, String), async_sqlite::Error> = get_lastfm_session_data(db_client, author.id.get()).await;
    if get_session_data.is_err() {
        error!("Failed to get {}'s session: {}", author.name, get_session_data.unwrap_err().to_string());
        ctx.send(poise::CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .description("Failed to get your session. Are you sure you've authorized before?\nTry again later or regenerate key with `/lastfm authorize`")
                    .color(Color::from_rgb(255, 100, 100))
            )
            .ephemeral(true)
        ).await?;

        return Ok(());
    }

    let (_session_key, session_username) = get_session_data.unwrap();

    let lastfm: &Lastfm = &custom_data.lastfm_client;

    let get_recents = lastfm.user().get_recent_tracks().limit(1).username(&session_username).send().await;

    if get_recents.is_err() {
        error!("Failed to get {}'s last.fm playing track: {}", author.id.get(), get_recents.unwrap_err().to_string());
        ctx.send(poise::CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .description("Failed to get your playing track. Please try again later, if the issue persists contact <@908779319084589067>")
                    .color(Color::from_rgb(255, 100, 100))
            )
            .ephemeral(true)
        ).await?;

        return Ok(());
    }

    let recents_response = get_recents.unwrap();//["recenttracks"]
    let recents;

    match recents_response {
        APIResponse::Success(real_recents) => {
            recents = real_recents;
        },
        APIResponse::Error(_) => { return Ok(()); }, // TODO: I am not sure if it can return error, but if it can do that add handle to that later
    }

    // TODO: Check if recenttracks.track len is more than 0 (last played exists and check for "nowplaying" in @attr)

    let last_track = &recents["recenttracks"]["track"][0];
    let last_track_thumbnail = last_track["image"][3]["#text"].as_str().unwrap();
    let last_track_artist = last_track["artist"]["#text"].as_str().unwrap();
    let last_track_name = last_track["name"].as_str().unwrap();
    let last_track_url = last_track["url"].as_str().unwrap();

    ctx.send(poise::CreateReply::default()
        .embed(
            CreateEmbed::default()
                .thumbnail(last_track_thumbnail)
                .title(format!("{} — {}", last_track_artist, last_track_name))
                .url(last_track_url)
                .description("very cool track")
                .color(Color::from_rgb(255, 255, 255))
        )
    ).await?;

    Ok(())
}