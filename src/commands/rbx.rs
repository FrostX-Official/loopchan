use std::time::Duration;

use poise::CreateReply;
use roboat::{thumbnails::{ThumbnailSize, ThumbnailType}, users::UsernameUserDetails};
use serenity::all::{ButtonStyle, Colour, CreateActionRow, CreateButton, CreateEmbed};

use crate::{utils::{basic::remove_whitespace, database::linking::get_roblox_id_in_users_db_by_discord_id}, Context, Data, Error};

/// Verify your Discord account by linking it to your Roblox account
#[poise::command(slash_command, global_cooldown=2, user_cooldown=10)]
pub async fn verify(
    ctx: Context<'_>,
    #[min_length = 2] #[description = "Roblox Username"] roblox_username: Option<String>,
    #[description = "Roblox User ID"] roblox_user_id: Option<u64>
) -> Result<(), Error>  {
    // Check if user already has roblox_id in db
    let ctx_data: &Data = ctx.data();
    let roblox_client: &roboat::Client = &ctx_data.roblox_client;
    let db_client: &async_sqlite::Client = &ctx_data.db_client;  
    let author_id: u64 = ctx.author().id.get();
    let roblox_id_in_db: Result<u64, async_sqlite::Error> = get_roblox_id_in_users_db_by_discord_id(db_client, author_id).await;
    
    if !roblox_id_in_db.is_ok() { // Fail-check
        ctx.send(poise::CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .title("An error occurred.")
                    .description(format!("Failed to find your data in database! Please try again later or report this issue to <@908779319084589067>!\n-# {}", roblox_id_in_db.err().unwrap().to_string()))
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
                        .description(format!("You're already verified!\n-# But loopchan failed to find info about your linked account... ? huh.. T-T\n-# {}", roblox_id_in_db_unwrapped))
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
                        .description(format!("You're already verified as **@{}**!\n-# {}", user_details_unwrapped.username, roblox_id_in_db_unwrapped))
                        .color(Colour::from_rgb(255, 80, 80))
                )
                .ephemeral(true)
            ).await?;
        } else {
            ctx.send(poise::CreateReply::default()
                .embed(
                    CreateEmbed::default()
                        .title("Already verified!")
                        .description(format!("You're already verified as **@{}**!\n-# {}", user_details_unwrapped.username, roblox_id_in_db_unwrapped))
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
    ]);

    let builder: poise::CreateReply = poise::CreateReply::default()
        .embed(
            CreateEmbed::default()
                .title("Found User!")
                .description(
                    format!("Username: {}\nUser ID: {}\n**Please confirm that this is your Roblox Account by changing your profile description to:**\n```{}```\n## You have 5 minutes.\n-# You can change it back after verification process! (Make sure to save it though :D)", roblox_username, roblox_user_id, randomwords.join("\n"))
                )
                .color(Colour::from_rgb(255, 255, 100))
        )
        .components(vec![components])
        .ephemeral(true);

    randomwords.remove(0); // For some reason if you join vector with \n separator it will not show first element in embed. This is why we're deleting it after creating embed
    let no_whitespace_wordgen = remove_whitespace(&randomwords.join("\n"));

    ctx_data.verifications.lock().await.insert(author_id, (no_whitespace_wordgen, roblox_user_id));
    let reply = ctx.send(builder).await?;

    let interaction = reply
        .message()
        .await?
        .await_component_interaction(ctx)
        .author_id(ctx.author().id)
        .timeout(Duration::new(300, 0))
        .await;

    if interaction.is_none() {
        reply
            .edit(
                ctx,
                CreateReply::default()
                    .components(vec![])
                    .content("Timed out.")
                    .ephemeral(true)
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