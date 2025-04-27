use std::time::Duration;

use roboat::{thumbnails::{ThumbnailSize, ThumbnailType}, users::UsernameUserDetails};
use serenity::all::{ButtonStyle, Colour, CreateActionRow, CreateButton, CreateEmbed};

use crate::{utils::db::get_roblox_id_in_db_by_discord_id, Context, Error};

fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Verify your Discord account by linking it to your Roblox account
#[poise::command(slash_command, global_cooldown=1, user_cooldown=5)]
pub async fn verify(
    ctx: Context<'_>,
    #[min_length = 2] #[description = "Roblox Username"] roblox_username: Option<String>,
    #[description = "Roblox User ID"] roblox_user_id: Option<u64>
) -> Result<(), Error>  {
    // Check if user already has roblox_id in db
    let roblox_client: &roboat::Client = &ctx.data().roblox_client;
    let db_client: &async_sqlite::Client = &ctx.data().db_client;
    let roblox_id_in_db: Result<u64, async_sqlite::Error> = get_roblox_id_in_db_by_discord_id(db_client, ctx.author().id.get()).await;
    
    if !roblox_id_in_db.is_ok() { // Fail-check
        ctx.send(poise::CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .title("An error occurred.")
                    .description("Failed to find your data in database! Please try again later or report this issue to <@908779319084589067>!\n-# ".to_owned()+&roblox_id_in_db.err().unwrap().to_string())
                    .color(Colour::from_rgb(255, 80, 80))
            )
            .ephemeral(true)
        ).await?;
        return Ok(());
    }

    let roblox_id_in_db_unwrapped: u64 = roblox_id_in_db.unwrap();

    if roblox_id_in_db_unwrapped != 0 {
        let user_details: Result<roboat::users::UserDetails, roboat::RoboatError> = roblox_client.user_details(roblox_id_in_db_unwrapped).await;
        if !user_details.is_ok() {
            ctx.send(poise::CreateReply::default()
                .embed(
                    CreateEmbed::default()
                        .title("Already verified!")
                        .description("You're already verified!\n-# But loopchan failed to find info about your linked account... ? huh.. T-T\n-# ".to_owned()+&roblox_id_in_db_unwrapped.to_string())
                        .color(Colour::from_rgb(255, 80, 80))
                )
                .ephemeral(true)
            ).await?;
            return Ok(());
        }

        let user_details_unwrapped: roboat::users::UserDetails = user_details.unwrap();

        let headshot_thubmnail: Result<String, roboat::RoboatError> = roblox_client.thumbnail_url(roblox_id_in_db_unwrapped, ThumbnailSize::S420x420, ThumbnailType::AvatarHeadshot).await;
        if headshot_thubmnail.is_ok() {
            ctx.send(poise::CreateReply::default()
                .embed(
                    CreateEmbed::default()
                        .title("Already verified!")
                        .thumbnail(headshot_thubmnail.unwrap())
                        .description("You're already verified as **@".to_owned()+&user_details_unwrapped.username+"**!\n-# "+&roblox_id_in_db_unwrapped.to_string())
                        .color(Colour::from_rgb(255, 80, 80))
                )
                .ephemeral(true)
            ).await?;
        } else {
            ctx.send(poise::CreateReply::default()
                .embed(
                    CreateEmbed::default()
                        .title("Already verified!")
                        .description("You're already verified as **@".to_owned()+&user_details_unwrapped.username+"**!\n-# "+&roblox_id_in_db_unwrapped.to_string())
                        .color(Colour::from_rgb(255, 80, 80))
                )
                .ephemeral(true)
            ).await?;
        }
        return Ok(());
    }

    // User hasn't provided Roblox Username, neither Roblox User ID
    if roblox_username.is_none() & roblox_user_id.is_none() {
        ctx.send(poise::CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .title("An error occurred.")
                    .description("Please provide either `Roblox Username` or `Roblox User ID`\n-# (not both, atleast one of them!)")
                    .color(Colour::from_rgb(255, 80, 80))
            )
            .ephemeral(true)
        ).await?;
        return Ok(());
    }

    // First try to get user details, if they weren't provided try to get username user details.
    let user_details: Option<Result<roboat::users::UserDetails, roboat::RoboatError>>;
    let username_details: Option<Result<Vec<roboat::users::UsernameUserDetails>, roboat::RoboatError>>;

    if !roblox_user_id.is_none() {
        user_details = Some(roblox_client.user_details(roblox_user_id.unwrap()).await);
        username_details = None;
    } else {
        username_details = Some(roblox_client.username_user_details(vec![roblox_username.expect("")], true).await);
        user_details = None;
    }

    // Check if both are failed to be fetched.

    let user_details_ref: Option<&Result<roboat::users::UserDetails, roboat::RoboatError>> = user_details.as_ref();
    let username_details_ref: Option<&Result<Vec<roboat::users::UsernameUserDetails>, roboat::RoboatError>> = username_details.as_ref();
    
    let mut user_details_failed: bool = true;
    if !user_details_ref.is_none() {
        user_details_failed = !user_details_ref.unwrap().is_ok();
    }
    let mut username_details_failed: bool = true;
    if !username_details_ref.is_none() {
        username_details_failed = !username_details_ref.unwrap().is_ok();
    }
    
    if user_details_failed & username_details_failed {
        ctx.send(poise::CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .title("An error occurred.")
                    .description("Failed to find your Roblox account. Please try again later or report this issue to <@908779319084589067>!")
                    .color(Colour::from_rgb(255, 80, 80))
            )
            .ephemeral(true)
        ).await?;
        return Ok(());
    }

    // Get final user information before asking user to change their profile description to wordgen.

    // You might ask: "Why not just make 1 variable for both types of user details?"
    // And the answer to this would be that for some fucking reason Username User Details doesn't have `description` and it's separated from base user details.
    // Also that would be pointless since we don't need description before changing user changing it to wordgen (should we save their description incase they fail to copy it manually and forget?)
    // But we need username to get representation of user and id to get headshot thumbnail.
    // don't mind this yap, forget all I said above hehe :3

    let roblox_user_id: u64;
    let roblox_username: String;

    if !user_details_failed {
        let unwrapped_details = user_details.unwrap().unwrap();
        roblox_user_id = unwrapped_details.id;
        roblox_username = unwrapped_details.username;
    } else {
        let unwrapped_details = username_details.unwrap().unwrap();
        let actual_details: Option<&UsernameUserDetails> = unwrapped_details.first();
        if actual_details.is_none() {
            ctx.send(poise::CreateReply::default()
                .embed(
                    CreateEmbed::default()
                        .title("An error occurred.")
                        .description("Failed to find your Roblox account. Please try again later or report this issue to <@908779319084589067>!")
                        .color(Colour::from_rgb(255, 80, 80))
                )
                .ephemeral(true)
            ).await?;
            return Ok(());
        } else {
            let unwrapped_actual_details = actual_details.unwrap();
            roblox_user_id = unwrapped_actual_details.id;
            roblox_username = unwrapped_actual_details.username.clone();
        }
    }

    // Generate wordgen and ask user to change their profile description to it. (and then user clicks done or cancel buttons)

    let mut randomwords: Vec<String> = vec![];
    for _ in 0..11 {
        randomwords.insert(0, crate::utils::wordgen::getrandomgenword().await);
    }

    let components: CreateActionRow = CreateActionRow::Buttons(vec![
        CreateButton::new("verification.check")
            .label("Done")
            .style(ButtonStyle::Primary)
            .emoji('✅'),
        CreateButton::new("verification.cancel")
            .label("Cancel")
            .style(ButtonStyle::Secondary)
            .emoji('❌'),
        CreateButton::new("verification.regenerate")
            .label("Regenerate")
            .style(ButtonStyle::Secondary)
            .emoji('🔃')
            .disabled(true), // TODO: Add random words regeneration functionality later (incase the ones that were generated before are censored by Roblox)
    ]);

    let builder: poise::CreateReply = poise::CreateReply::default()
        .embed(
            CreateEmbed::default()
                .title("Found User!")
                .description(
                    "Username: ".to_owned()+&roblox_username+"\nUser ID: "+&roblox_user_id.to_string()+
                    "\n**Please confirm that this is your Roblox Account by changing your profile description to:**\n```"
                    +&randomwords.join("\n")
                    +"```\n## You have 5 minutes.\n-# You can change it back after verification process! (Make sure to save it though :D)"
                )
                .color(Colour::from_rgb(255, 255, 100))
        )
        .components(vec![components])
        .ephemeral(true);

    randomwords.remove(0); // For some reason if you join vector with \n separator it will not show first element in embed. This is why we're deleting it after creating embed
    let no_whitespace_wordgen = remove_whitespace(&randomwords.join("\n"));

    let reply = ctx.send(builder).await?;

    let interaction = reply
        .message()
        .await?
        .await_component_interaction(ctx)
        .author_id(ctx.author().id)
        .timeout(Duration::new(300, 0))
        .await;

    reply
        .edit(
            ctx,
            poise::CreateReply::default()
                .components(vec![])
                .content("Processing... Please wait."),
        )
        .await?;
    
    let pressed_button_id = match &interaction {
        Some(m) => &m.data.custom_id,
        None => {
            ctx.say("⚠ You didn't interact in time!\nRun the command again if you want to verify.").await?;
            return Ok(());
        }
    };

    if pressed_button_id == "verification.cancel" {
        // Cancel verification
        reply
            .edit(
                ctx,
                poise::CreateReply::default()
                    .content("❌ Verification Cancelled."),
            )
            .await?;
    } else {
        // Check if wordgens match
        let user_details_fetch: Result<roboat::users::UserDetails, roboat::RoboatError> = roblox_client.user_details(roblox_user_id).await;
        if !user_details_fetch.is_ok() {
            reply
                .edit(
                    ctx,
                    poise::CreateReply::default()
                        .content("Failed to verify your account!\nPlease try again later."),
                )
                .await?;
            return Ok(());
        }

        let user_details_fetch_unwrapped: roboat::users::UserDetails = user_details_fetch.unwrap();
        let user_description: String = user_details_fetch_unwrapped.description;
        let no_whitespace_description = remove_whitespace(&user_description);
        
        if no_whitespace_description != no_whitespace_wordgen {
            reply
                .edit(
                    ctx,
                    poise::CreateReply::default()
                        .content("Your Roblox profile description does not match wordgen.\nIf you think that's not true contact <@908779319084589067> for support!\nYou can try again later."),
                )
                .await?;
            return Ok(());
        }

       // TODO: Change user's roblox_id in db to new, verified one (also update roles depending on data in game)

        reply
            .edit(
                ctx,
                poise::CreateReply::default()
                    .content("Work in progress."),
            )
            .await?;
    }

    Ok(())
}

/// Fetches roblox's account data
#[poise::command(slash_command)]
pub async fn fetchdata(
    ctx: Context<'_>,
    #[description = "Roblox User ID"] roblox_user_id: u64,
) -> Result<(), Error> {
    let roblox_client: &roboat::Client = &ctx.data().roblox_client;

    let roblox_user: Result<roboat::users::UserDetails, roboat::RoboatError> = roblox_client.user_details(roblox_user_id).await;
    if roblox_user.is_ok() {
        let roblox_user: roboat::users::UserDetails = roblox_user.unwrap();
        // Try to fetch headshot thumbnail, display embed with it if succeded.
        let headshot_thubmnail = roblox_client.thumbnail_url(roblox_user.id, ThumbnailSize::S420x420, ThumbnailType::AvatarHeadshot).await;
        if headshot_thubmnail.is_ok() {
            ctx.send(poise::CreateReply::default()
                .embed(
                    CreateEmbed::default()
                        .title("✅ User Found!")
                        .thumbnail(headshot_thubmnail.unwrap())
                        .field("User ID", roblox_user.id.to_string(), false)
                        .field("Display Name", roblox_user.display_name, false)
                        .field("Username", roblox_user.username, false)
                        .color(Colour::from_rgb(80, 255, 80))
                )
                //.ephemeral(true)  
            ).await?;
        } else {
            ctx.send(poise::CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .title("✅ User Found!")
                    .field("User ID", roblox_user.id.to_string(), false)
                    .field("Display Name", roblox_user.display_name, false)
                    .field("Username", roblox_user.username, false)
                    .color(Colour::from_rgb(80, 255, 80))
            )
            //.ephemeral(true)
        ).await?;
        }
    } else {
        ctx.send(poise::CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .title("❌ User Not Found!")
                    .color(Colour::from_rgb(255, 80, 80))
            )
            .ephemeral(true)
        ).await?;
    }
    Ok(())
}