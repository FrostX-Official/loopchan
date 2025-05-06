use std::time::Duration;

use serenity::all::{ButtonStyle, Color, CreateActionRow, CreateButton, CreateEmbed};
use tracing::{error, warn};
use crate::{Context, Error};

use lastfm_rust::Lastfm;
use lastfm_rust::APIResponse;

/// Last.fm API Commands
#[poise::command(slash_command, subcommands("authorize"), subcommand_required)]
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
    let custom_data: &crate::Data = ctx.data();
    let lastfm: &Lastfm = &custom_data.lastfm_client;
    let token_response: Result<String, lastfm_rust::Error> = generate_token(lastfm).await;

    if token_response.is_err() {
        ctx.send(poise::CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .description("Failed to generate a token for you! Please try again later or report this issue to <@908779319084589067>")
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
                .description("**indev**\npress `Done` button after confirmed `Connect` on Last.fm website.\n(you have 5 minutes or less (i forgor))")
                .color(Color::from_rgb(255, 255, 255))
        )
        .components(vec![ // nesting hell
            CreateActionRow::Buttons(vec![
                CreateButton::new_link(auth_url)
                    .label("Connect")
                    .style(ButtonStyle::Primary),
                CreateButton::new(format!("lastfm.authorized.{}.{}", ctx.author().id.get(), token))
                    .label("Done")
                    .style(ButtonStyle::Secondary),
                CreateButton::new("lastfm.cancel_auth")
                    .label("Cancel")
                    .style(ButtonStyle::Secondary)
        ])])
        .ephemeral(true)
    ).await;

    if auth_reply.is_err() {
        error!("Failed to send reply to user ({}) with auth URL: {}", ctx.author().id.get(), auth_reply.err().unwrap().to_string()); // idk why unwrap_err() doesn't work here T-T
        return Ok(());
    }

    // TODO: Move lastfm interaction handling to events handlers

    let reply: poise::ReplyHandle<'_> = auth_reply.unwrap();
    let interaction_not_timed_out = reply
        .message()
        .await?
        .await_component_interaction(ctx)
        .author_id(ctx.author().id)
        .timeout(Duration::new(300, 0))
        .await;

    if interaction_not_timed_out.is_none() {
        reply
            .edit(
                ctx,
                poise::CreateReply::default()
                    .components(vec![])
                    .content("Timed out."),
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
                    .content("Cancelled authorization."),
            )
            .await?;
        return Ok(());
    }

    let get_session_result = lastfm.auth().get_session().token(&token).send().await;
    if get_session_result.is_err() {
        error!("Failed to claim user's ({}) session key: {}", ctx.author().id.get(), get_session_result.unwrap_err().to_string()); 
        reply
            .edit(
                ctx,
                poise::CreateReply::default()
                    .content("Failed to claim your session key. Please try again later, if the issue persists contact <@908779319084589067>"),
            )
            .await?;
        return Ok(());
    }

    let session_data = &get_session_result.unwrap()["session"];
    // TODO: Store user's session key in db
    warn!("{}'s last.fm session key: {} / username: {}", ctx.author().name, session_data["key"], session_data["name"]);

    Ok(())
}