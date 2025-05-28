// TODO: Actual fishing (mayhaps also a minigame for better fish?)

use futures::{Stream, StreamExt};
use serenity::all::CreateEmbed;
use tracing::error;
use uuid::Uuid;

use crate::{utils::{basic::{fish_from_name, get_fishes_names_from_fishes}, database::fishing::{get_user_fishes_in_fishing_db, give_fish_to_user_in_fishing_db}}, Context, DataFish, Error};

/// Fishing Commands
#[poise::command(slash_command, subcommands("give_fish", "inventory"), subcommand_required)]
pub async fn fishing(_ctx: Context<'_>) -> Result<(), Error> { Ok(()) }

#[poise::command(slash_command)]
pub async fn inventory(
    ctx: Context<'_>
) -> Result<(), Error> {
    let custom_data = ctx.data();
    let db_client: &async_sqlite::Client = &custom_data.db_client;
    let inventory: Result<Vec<DataFish>, async_sqlite::Error> = get_user_fishes_in_fishing_db(db_client, ctx.author().id.get()).await;
    if inventory.is_err() { // TODO: Handle properly
        ctx.send(poise::CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .description("Failed")
            )
            .ephemeral(true)
        ).await?;
        return Ok(());
    }
    let inventory: Vec<DataFish> = inventory.unwrap();
    let inventory_size = inventory.len();

    let mut response: String = String::new();
    let mut index: u32 = 0;
    for fish in inventory {
        index+=1;
        let actual_fish = fish_from_name(&fish.r#type, custom_data.config.economy.fishes.clone()).unwrap();
        let mut final_string = format!(
            "\n{} • {}cm *(~${})*\n-# *{}*\n*{}*",
            fish.r#type, fish.size, (actual_fish.base_value as f64)*(fish.size as f64), fish.uuid, actual_fish.description // TODO: Move calculating value in money to other function and add modifiers possibility to it
        );
        if index != inventory_size as u32 {
            final_string.push_str("\n**==================================**");
        }
        response.push_str(&final_string);
    }

    ctx.send(poise::CreateReply::default()
        .embed(
            CreateEmbed::default()
                .title("🎣 Inventory")
                .description(response)
        )
    ).await?;

    Ok(())
}

async fn fish_type_autocomplete_handler<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let fish_names = get_fishes_names_from_fishes(ctx.data().config.economy.fishes.clone()); // TODO: Cache this in data to make autocomplete faster

    futures::stream::iter(fish_names)
        .filter(move |name| futures::future::ready(name.starts_with(partial)))
        .map(|name| name.to_string())
}

#[poise::command(slash_command)]
pub async fn give_fish(
    ctx: Context<'_>,
    #[autocomplete = "fish_type_autocomplete_handler"]
    r#type: String,
    size: f32
) -> Result<(), Error> {
    let db_client: &async_sqlite::Client = &ctx.data().db_client;
    let successfully_gave_fish: Result<usize, async_sqlite::Error> = give_fish_to_user_in_fishing_db(db_client, ctx.author().id.get(), DataFish {
        uuid: Uuid::new_v4().to_string(),
        modifiers: String::new(), // TODO: Make ability to modify fish or/and give fish with modifiers from the start
        r#type,
        size
    }).await;

    if successfully_gave_fish.is_err() {
        error!("failed to give fish: {}", successfully_gave_fish.unwrap_err().to_string());
        ctx.send(poise::CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .description("failed to give fish :(")
            )
            .ephemeral(true)
        ).await?;
        return Ok(());
    }

    ctx.send(poise::CreateReply::default()
        .embed(
            CreateEmbed::default()
                .description("gave fish :D")
        )
    ).await?;

    Ok(())
}