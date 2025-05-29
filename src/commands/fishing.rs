// TODO: Actual fishing (mayhaps also a minigame for better fish?)

use std::time::Duration;

use futures::{Stream, StreamExt};
use poise::{CooldownConfig, CreateReply};
use serenity::{all::{Color, CreateEmbed}, json};
use tracing::{error, warn};
use uuid::Uuid;

use crate::{utils::{basic::{fish_from_name, fishmodifiers_from_datafishmodifiers, get_fishes_names_from_fishes, remove_whitespace}, database::fishing::{get_user_fishes_in_fishing_db, give_fish_to_user_in_fishing_db}}, Context, DataFish, Error};

/// Fishing Commands
#[poise::command(slash_command, subcommands("give_fish", "inventory", "fish"), subcommand_required)]
pub async fn fishing(_ctx: Context<'_>) -> Result<(), Error> { Ok(()) }

/// See your inventory with fishes
#[poise::command(slash_command)]
pub async fn inventory(
    ctx: Context<'_>
) -> Result<(), Error> {
    let custom_data = ctx.data();
    let db_client: &async_sqlite::Client = &custom_data.db_client;
    let inventory: Result<Vec<DataFish>, async_sqlite::Error> = get_user_fishes_in_fishing_db(db_client, ctx.author().id.get()).await;

    if inventory.is_err() { // TODO: Handle properly
        ctx.send(CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .description("Failed")
                    .color(Color::from_rgb(255, 100, 100))
            )
            .ephemeral(true)
        ).await?;

        return Ok(());
    }

    let inventory: Vec<DataFish> = inventory.unwrap();
    let inventory_size = inventory.len();

    if inventory_size == 0 {
        ctx.send(CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .title("🎣 Inventory")
                    .description("Your inventory is empty 😥\nYou can catch fish by using command ```/fishing fish```")
                    .color(Color::from_rgb(255, 100, 100))
            )
        ).await?;

        return Ok(());
    }

    let mut response: String = String::new();
    let mut index: u32 = 0;
    for fish in inventory {
        index+=1;
        let actual_fish = fish_from_name(&fish.r#type, custom_data.config.economy.fishes.clone()).unwrap();

        let modifiers: Result<Vec<crate::FishModifier>, std::io::Error> = fishmodifiers_from_datafishmodifiers(fish.modifiers, custom_data.config.economy.fishes_modifiers.clone());
        if modifiers.is_err() {
            error!("Failed to decode modifiers of fish {}: {}", fish.uuid, modifiers.unwrap_err().to_string());
            ctx.send(CreateReply::default()
                .embed(
                    CreateEmbed::default()
                        .description(format!("Failed to decode modifiers of fish `{}`", fish.uuid))
                        .color(Color::from_rgb(255, 100, 100))
                )
                .ephemeral(true)
            ).await?;

            return Ok(());
        }

        let mut final_size: f32 = fish.size;
        let mut final_value: f64 = actual_fish.base_value as f64;

        let modifiers: Vec<crate::FishModifier> = modifiers.unwrap();
        let mut modifiers_formatted = String::new();
        if modifiers.len() > 0 {
            modifiers_formatted.push_str("\n**Modifier(s):**");
            for modifier in modifiers {
                if modifier.size_multiplier.is_some() {
                    final_size *= modifier.size_multiplier.unwrap()
                }

                if modifier.value_multiplier.is_some() {
                    final_value *= modifier.value_multiplier.unwrap() as f64
                }
                
                modifiers_formatted.push_str(&format!("\n**{}** • *1 in {}* • *{}*", modifier.name, modifier.chance, modifier.description));
            }
        }

        final_size = (final_size*100.0).floor()/100.0;
        final_value = (final_value*final_size as f64).floor();

        let mut final_string = format!(
            "\n### {} • {}cm *(~${})*\n-# ID: *{}*\nDescription: *{}*{}",
            fish.r#type, final_size, final_value, fish.uuid, actual_fish.description, modifiers_formatted
        );
        if index != inventory_size as u32 {
            final_string.push_str("\n\n**==================================**");
        }
        response.push_str(&final_string);
    }

    ctx.send(CreateReply::default()
        .embed(
            CreateEmbed::default()
                .title("🎣 Inventory")
                .description(response)
                .color(Color::from_rgb(255, 255, 255))
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
    #[description = "Select type only from autocomplete (otherwise datalose (haha. get it? datalose!))"]
    #[autocomplete = "fish_type_autocomplete_handler"]
    r#type: String,
    size: f32,
    #[description = "Fish Modifiers (separated by comma! (\",\"))"]
    modifiers: Option<String>,
) -> Result<(), Error> {
    let actual_fish: Result<crate::Fish, std::io::Error> = fish_from_name(&r#type, ctx.data().config.economy.fishes.clone());
    if actual_fish.is_err() {
        error!("failed to give fish: {}", actual_fish.unwrap_err().to_string());
        ctx.send(CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .description("failed to gib fish :(")
            )
            .ephemeral(true)
        ).await?;
        return Ok(());
    }
    let actual_fish: crate::Fish = actual_fish.unwrap();

    let mut modifiers_serialized: String = String::new();
    if modifiers.is_some() {
        let modifiers_unwrapped = remove_whitespace(&modifiers.unwrap());
        let modifiers_split = modifiers_unwrapped.split(",");
        let mut modifiers_vec = vec![];
        for modifier in modifiers_split {
            if !actual_fish.possible_modifiers.contains(&modifier.to_string()) {
                warn!("Tried to give fish with impossible modifier: {} to {}", modifier, r#type);
                continue;
            }
            modifiers_vec.push(modifier);
        }
        
        modifiers_serialized = json::to_string(&modifiers_vec).unwrap();
    }

    let db_client: &async_sqlite::Client = &ctx.data().db_client;
    let successfully_gave_fish: Result<usize, async_sqlite::Error> = give_fish_to_user_in_fishing_db(db_client, ctx.author().id.get(), DataFish {
        uuid: Uuid::new_v4().to_string(),
        modifiers: modifiers_serialized, // TODO: Make ability to modify fish or/and give fish with modifiers from the start
        r#type,
        size
    }).await;

    if successfully_gave_fish.is_err() {
        error!("failed to give fish: {}", successfully_gave_fish.unwrap_err().to_string());
        ctx.send(CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .description("failed to gib fish :(")
            )
            .ephemeral(true)
        ).await?;
        return Ok(());
    }

    ctx.send(CreateReply::default()
        .embed(
            CreateEmbed::default()
                .description("gave fish :D")
        )
    ).await?;

    Ok(())
}

/// Catch a fish!
#[poise::command(slash_command)]
pub async fn fish(
    ctx: Context<'_>
) -> Result<(), Error> {
    let economy_config: &crate::EconomyConfig = &ctx.data().config.economy;
    let on_cooldown: i32;

    {
        let mut cooldown_tracker = ctx.command().cooldowns.lock().unwrap();

        let mut cooldown_durations: CooldownConfig = CooldownConfig::default();
        cooldown_durations.user = Some(Duration::from_secs(economy_config.fish_cooldown*60));

        match cooldown_tracker.remaining_cooldown(ctx.cooldown_context(), &cooldown_durations) {
            Some(remaining) => {
                on_cooldown = remaining.as_secs() as i32;
            }
            None => {
                cooldown_tracker.start_cooldown(ctx.cooldown_context());
                on_cooldown = -1;
            },
        }
    };

    if on_cooldown != -1 {
        if on_cooldown > 60 {
            ctx.send(
                CreateReply::default()
                    .embed(
                        CreateEmbed::default()
                            .description(format!("You're currently too exhausted to fish! Wait `{}` minutes.\n-# {} seconds", on_cooldown/60, on_cooldown))
                            .color(Color::from_rgb(255, 100, 100))
                    )
                    .ephemeral(true)
            ).await?;

            return Ok(());
        }

        ctx.send(
            CreateReply::default()
                .embed(
                    CreateEmbed::default()
                        .description(format!("You're currently too exhausted to fish! Wait `{}` seconds.", on_cooldown))
                        .color(Color::from_rgb(255, 100, 100))
                )
                .ephemeral(true)
        ).await?;

        return Ok(());
    }

    // TODO: Actual fishing (mayhaps also a minigame for better fish?)

    ctx.send(CreateReply::default()
        .embed(
            CreateEmbed::default()
                .description("wip")
        )
    ).await?;

    Ok(())
}