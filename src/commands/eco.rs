use crate::{Context, Error};

use serenity::all::{Color, CreateEmbed};
use tracing::{warn, info};

use crate::utils::db::{create_user_in_eco_db, get_user_balance_in_eco_db, get_user_level_and_experience_in_eco_db, update_user_level_and_experience_in_eco_db};

const LEVEL_PROGRESSBAR_SIZE: u64 = 18; // Progressbar: "[------------------]"

fn exp_needed_to_next_level(level: u64) -> u64 {
    return (((level * 100) as f64) * 1.25).ceil() as u64;
}

pub async fn handle_user_exp_update(
    db_client: &async_sqlite::Client, // db client to index economics in
    userid: u64, // User ID to index in db
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
    user: serenity::model::user::User,
    amount: u64
) {
    let userid: u64 = user.id.get();
    let successfully_created: Result<usize, async_sqlite::Error> = create_user_in_eco_db(&custom_data.db_client, userid).await; // Why can't I put ? here to ignore Result???
    if successfully_created.is_err() {
        warn!("Failed to create user ({}) in eco db: {}", userid, successfully_created.unwrap_err().to_string());
        return;
    }

    let level_exp_check: Result<(Result<u64, async_sqlite::rusqlite::Error>, Result<u64, async_sqlite::rusqlite::Error>), async_sqlite::Error> = get_user_level_and_experience_in_eco_db(&custom_data.db_client, userid).await;
    
    if !level_exp_check.is_ok() {
        warn!("Failed to check {}'s level and experience: {}", userid, level_exp_check.unwrap_err().to_string());
        return;
    }

    let level_and_exp_checks: (Result<u64, async_sqlite::rusqlite::Error>, Result<u64, async_sqlite::rusqlite::Error>) = level_exp_check.unwrap();

    if !level_and_exp_checks.0.is_ok() {
        warn!("Failed to check {}'s level: {}", userid, level_and_exp_checks.0.unwrap_err().to_string());
        return;
    }

    if !level_and_exp_checks.1.is_ok() {
        warn!("Failed to check {}'s experience: {}", userid, level_and_exp_checks.1.unwrap_err().to_string());
        return;
    }

    let level: u64 = level_and_exp_checks.0.unwrap();
    let mut experience: u64 = level_and_exp_checks.1.unwrap();

    experience += amount;

    let successfully_updated: Result<usize, async_sqlite::Error> = handle_user_exp_update(&custom_data.db_client, userid, level, experience).await;
    if successfully_updated.is_err() {
        warn!("Failed to update user ({}) in eco db: {}", userid, successfully_updated.unwrap_err().to_string());
    }
}

/// Economics Commands
#[poise::command(slash_command, subcommands("balance", "level", "modify_data"), subcommand_required)]
pub async fn eco(_ctx: Context<'_>) -> Result<(), Error> { Ok(()) }

/// Modify user's level and/or experience.
#[poise::command(slash_command)]
pub async fn modify_data(
    ctx: Context<'_>,
    #[description = "Member"] user: Option<serenity::model::user::User>,
    #[description = "Level"] level: Option<u64>,
    #[description = "Experience"] experience: Option<u64>
) -> Result<(), Error> {
    if level.is_none() && experience.is_none() {
        ctx.send(poise::CreateReply::default()
            .content("Provide level and/or experience.")
            .ephemeral(true)
        ).await?;
        return Ok(());
    }

    let db_client: &async_sqlite::Client = &ctx.data().db_client;

    let nuser: serenity::model::user::User;
    if user.is_none() {
        nuser = ctx.author().clone();
    } else {
        nuser = user.unwrap();
    }
    let nuser_id: u64 = nuser.id.get();

    let successful: Result<usize, async_sqlite::Error>;
    if experience.is_some() {
        let actual_level: u64;

        if level.is_some() {
            actual_level = level.unwrap()
        } else {
            let level_exp_check: Result<(Result<u64, async_sqlite::rusqlite::Error>, Result<u64, async_sqlite::rusqlite::Error>), async_sqlite::Error> = get_user_level_and_experience_in_eco_db(db_client, nuser_id).await;
        
            if level_exp_check.is_err() {
                warn!("Failed to check {}'s level and experience: {}", nuser_id, level_exp_check.unwrap_err().to_string());
                ctx.send(poise::CreateReply::default()
                    .content("Failed to modify data! (check console~)")
                    .ephemeral(true)
                ).await?;
                return Ok(());
            }

            let level_and_exp_checks: (Result<u64, async_sqlite::rusqlite::Error>, Result<u64, async_sqlite::rusqlite::Error>) = level_exp_check.unwrap();

            if level_and_exp_checks.0.is_err() {
                warn!("Failed to check {}'s level: {}", nuser_id, level_and_exp_checks.0.unwrap_err().to_string());
                ctx.send(poise::CreateReply::default()
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
        ctx.send(poise::CreateReply::default()
            .content("Successful\n-# usize: ".to_owned()+&successful.unwrap().to_string())
            .ephemeral(true)
        ).await?;
    } else {
        warn!("Failed to modify data: {}", successful.err().unwrap().to_string());
        ctx.send(poise::CreateReply::default()
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

    let nuser: serenity::model::user::User;
    if user.is_none() {
        nuser = ctx.author().clone();
    } else {
        nuser = user.unwrap();
    }

    let nuser_id: u64 = nuser.id.into();

    create_user_in_eco_db(&custom_data.db_client, nuser_id).await?;
    let balance_check: Result<u64, async_sqlite::Error> = get_user_balance_in_eco_db(&custom_data.db_client, nuser_id).await;
    
    if !balance_check.is_ok() {
        warn!("Failed to check {}'s balance: {}", nuser_id, balance_check.unwrap_err().to_string());
        ctx.send(poise::CreateReply::default()
            .content("Failed to check user's balance! Please try again later or report this issue to <@908779319084589067>.")
            .ephemeral(true)
        ).await?;

        return Ok(());
    }

    let balance: u64 = balance_check.unwrap();

    ctx.send(poise::CreateReply::default()
        .content("<@".to_owned()+&nuser.id.to_string()+">'s Balance: "+&balance.to_string())
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

    let nuser: serenity::model::user::User;
    if user.is_none() {
        nuser = ctx.author().clone();
    } else {
        nuser = user.unwrap();
    }

    let nuser_id: u64 = nuser.id.into();

    create_user_in_eco_db(&custom_data.db_client, nuser_id).await?;
    let level_exp_check: Result<(Result<u64, async_sqlite::rusqlite::Error>, Result<u64, async_sqlite::rusqlite::Error>), async_sqlite::Error> = get_user_level_and_experience_in_eco_db(&custom_data.db_client, nuser_id).await;
    
    if !level_exp_check.is_ok() {
        warn!("Failed to check {}'s level and experience: {}", nuser_id, level_exp_check.unwrap_err().to_string());
        ctx.send(poise::CreateReply::default()
            .content("Failed to check user's level! Please try again later or report this issue to <@908779319084589067>.")
            .ephemeral(true)
        ).await?;

        return Ok(());
    }

    let level_and_exp_checks: (Result<u64, async_sqlite::rusqlite::Error>, Result<u64, async_sqlite::rusqlite::Error>) = level_exp_check.unwrap();

    if !level_and_exp_checks.0.is_ok() {
        warn!("Failed to check {}'s level: {}", nuser_id, level_and_exp_checks.0.unwrap_err().to_string());
        ctx.send(poise::CreateReply::default()
            .content("Failed to check user's level! Please try again later or report this issue to <@908779319084589067>.")
            .ephemeral(true)
        ).await?;

        return Ok(());
    }

    if !level_and_exp_checks.1.is_ok() {
        warn!("Failed to check {}'s experience: {}", nuser_id, level_and_exp_checks.1.unwrap_err().to_string());
        ctx.send(poise::CreateReply::default()
            .content("Failed to check user's level! Please try again later or report this issue to <@908779319084589067>.")
            .ephemeral(true)
        ).await?;

        return Ok(());
    }

    let level: u64 = level_and_exp_checks.0.unwrap();
    let experience: u64 = level_and_exp_checks.1.unwrap();
    let experience_needed: u64 = exp_needed_to_next_level(level);
    let progress: u64 = ((experience as f64)/(experience_needed as f64)*(LEVEL_PROGRESSBAR_SIZE as f64)).floor() as u64;
    let progressbar = "\n``[".to_owned()+(&"=".repeat(progress as usize))+(&"-".repeat((LEVEL_PROGRESSBAR_SIZE-progress) as usize))+"]``";

    ctx.send(poise::CreateReply::default()
        .embed(CreateEmbed::default()
            .title(nuser.name+"'s Level Info")
            .description("**Level:** ".to_owned()+&level.to_string()+
                        "\n**Experience:** "+&experience.to_string()+"/"+&experience_needed.to_string()+&progressbar)
            .color(Color::from_rgb(255, 255, 255))
        )
    ).await?;

    Ok(())
}