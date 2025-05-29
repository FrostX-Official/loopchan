use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures::{Stream, StreamExt};
use poise::{CooldownConfig, CreateReply};
use rand::{distr::{weighted::WeightedIndex, Distribution}, rng, Rng};
use serenity::{all::{ButtonStyle, Color, ComponentInteraction, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage}, json};
use tracing::{error, warn};
use uuid::Uuid;

use crate::{utils::{basic::{fish_from_name, fishmodifier_from_name, fishmodifiers_from_datafishmodifiers, get_fishes_names_from_fishes, remove_whitespace}, database::fishing::{get_user_fishes_in_fishing_db, give_fish_to_user_in_fishing_db}}, Context, DataFish, Error, FishModifier};

/// Fishing Commands
#[poise::command(slash_command, subcommands("give_fish", "inventory", "fish"), subcommand_required)]
pub async fn fishing(_ctx: Context<'_>) -> Result<(), Error> { Ok(()) }

pub fn get_inventory_components(
    current_page: u32,
    inventory_size: u32
) -> Vec<CreateActionRow> {
    let mut components = vec![CreateActionRow::Buttons(vec![])];

    let prev_visible = current_page > 0;
    let next_visible = (current_page+1)*5 < (inventory_size as u32);

    if prev_visible {
        match components.get(0).unwrap() {
            CreateActionRow::Buttons(current_buttons) => {
                let mut buttons = current_buttons.clone();
                buttons.push(
                    CreateButton::new(format!("fishing.inventory.prev.{current_page}"))
                        .label("◀")
                        .style(ButtonStyle::Secondary)
                );
                components = vec![CreateActionRow::Buttons(buttons)];
            },
            _ => {}
        }
    }
    if next_visible {
        match components.get(0).unwrap() {
            CreateActionRow::Buttons(current_buttons) => {
                let mut buttons = current_buttons.clone();
                buttons.push(
                    CreateButton::new(format!("fishing.inventory.next.{current_page}"))
                        .label("▶")
                        .style(ButtonStyle::Secondary)
                );
                components = vec![CreateActionRow::Buttons(buttons)];
            },
            _ => {}
        }
    }

    if !prev_visible & !next_visible {
        components = vec![];
    }

    components
}

pub async fn get_inventory_embeds_alt(
    ctx: &serenity::prelude::Context,
    interaction: &ComponentInteraction,
    data: &crate::Data,
    inventory: Vec<DataFish>,
    page: u32
) -> Option<Vec<CreateEmbed>> {
    let mut embeds: Vec<CreateEmbed> = vec![
        CreateEmbed::default()
            .title(format!("🎣 Inventory{}", if page != 0 { format!(" | Page {}", page+1) } else { "".to_string() }))
            .color(Color::from_rgb(255, 255, 255)) 
    ];

    let base_calc = (page+1)*5;
    let max_index = if base_calc > 0 { base_calc } else { 5 };

    let mut index: u32 = 0;
    for fish in inventory {
        index += 1;
        if index < page*5 {
            continue;
        }
        if index >= max_index {
            continue;
        }

        let actual_fish = fish_from_name(&fish.r#type, data.config.economy.fishes.clone()).unwrap();

        let modifiers: Result<Vec<crate::FishModifier>, std::io::Error> = fishmodifiers_from_datafishmodifiers(&fish.modifiers, data.config.economy.fishes_modifiers.clone());
        if modifiers.is_err() {
            error!("Failed to decode modifiers of fish {}: {}", fish.uuid, modifiers.unwrap_err().to_string());

            interaction.create_response(
                ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::default()
                        .embed(
                            CreateEmbed::default()
                                .description(format!("Failed to decode modifiers of fish `{}`", fish.uuid))
                                .color(Color::from_rgb(255, 100, 100))
                        )
                        .components(vec![])
                        .ephemeral(true)
                )
            ).await.unwrap();

            return None;
        }

        let mut final_size: f32 = fish.size;
        let mut final_value: f64 = actual_fish.base_value as f64;

        let modifiers: Vec<crate::FishModifier> = modifiers.unwrap();
        let mut modifiers_formatted = String::new();
        if modifiers.len() > 0 {
            modifiers_formatted.push_str("\n### Modifier(s):");
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

        let final_string = format!(
            "\n### {} • {}cm *(~${})*\n*{}*{}",
            fish.r#type, final_size, final_value, actual_fish.description, modifiers_formatted
        );
        embeds.push(
            CreateEmbed::default()
                .description(final_string)
                .footer(CreateEmbedFooter::new(fish.uuid))
                .color(Color::from(actual_fish.color))
        );
    }

    Some(embeds)
}

pub async fn get_inventory_embeds(
    ctx: Context<'_>,
    inventory: Vec<DataFish>,
    page: u32
) -> Option<Vec<CreateEmbed>> {
    let custom_data = ctx.data();
    let mut embeds: Vec<CreateEmbed> = vec![
        CreateEmbed::default()
            .title(format!("🎣 Inventory{}", if page != 0 { format!(" | Page {}", page+1) } else { "".to_string() }))
            .color(Color::from_rgb(255, 255, 255)) 
    ];

    let base_calc = (page+1)*5;
    let max_index = if base_calc > 0 { base_calc } else { 5 };

    let mut index: u32 = 0;
    for fish in inventory {
        index += 1;
        if index < page*5 {
            continue;
        }
        if index >= max_index {
            continue;
        }

        let actual_fish = fish_from_name(&fish.r#type, custom_data.config.economy.fishes.clone()).unwrap();

        let modifiers: Result<Vec<crate::FishModifier>, std::io::Error> = fishmodifiers_from_datafishmodifiers(&fish.modifiers, custom_data.config.economy.fishes_modifiers.clone());
        if modifiers.is_err() {
            error!("Failed to decode modifiers of fish {}: {}", fish.uuid, modifiers.unwrap_err().to_string());

            ctx.send(CreateReply::default()
                .embed(
                    CreateEmbed::default()
                        .description(format!("Failed to decode modifiers of fish `{}`", fish.uuid))
                        .color(Color::from_rgb(255, 100, 100))
                )
                .ephemeral(true)
            ).await.unwrap();

            return None;
        }

        let mut final_size: f32 = fish.size;
        let mut final_value: f64 = actual_fish.base_value as f64;

        let modifiers: Vec<crate::FishModifier> = modifiers.unwrap();
        let mut modifiers_formatted = String::new();
        if modifiers.len() > 0 {
            modifiers_formatted.push_str("\n### Modifier(s):");
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

        let final_string = format!(
            "\n### {} • {}cm *(~${})*\n*{}*{}",
            fish.r#type, final_size, final_value, actual_fish.description, modifiers_formatted
        );
        embeds.push(
            CreateEmbed::default()
                .description(final_string)
                .footer(CreateEmbedFooter::new(fish.uuid))
                .color(Color::from(actual_fish.color))
        );
    }

    Some(embeds)
}

/// See your inventory with fishes
#[poise::command(slash_command)] // TODO: Selling (and trading)
pub async fn inventory(
    ctx: Context<'_>
) -> Result<(), Error> {
    let custom_data = ctx.data();
    let db_client: &async_sqlite::Client = &custom_data.db_client;
    let inventory: Result<Vec<DataFish>, async_sqlite::Error> = get_user_fishes_in_fishing_db(db_client, ctx.author().id.get()).await;

    if inventory.is_err() {
        error!("Failed to get {}'s fishes: {}", ctx.author().id.get(), inventory.unwrap_err().to_string());

        ctx.send(CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .description("Failed to find your fishes! Please try again later, if the issue persists contact <@908779319084589067>")
                    .color(Color::from_rgb(255, 100, 100))
            )
            .ephemeral(true)
        ).await?;

        ctx.set_invocation_data(true).await; // cancel cooldown (hopefully)

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

    let current_page: u32 = 0;
    let embeds: Option<Vec<CreateEmbed>> = get_inventory_embeds(ctx, inventory, current_page).await;

    if embeds.is_none() {
        ctx.send(CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .description("Failed to find your fishes! Please try again later, if the issue persists contact <@908779319084589067>")
                    .color(Color::from_rgb(255, 100, 100))
            )
            .ephemeral(true)
        ).await?;

        return Ok(());
    }

    let components = get_inventory_components(current_page, inventory_size as u32);

    let mut createreply = CreateReply::default();
    createreply.embeds = embeds.unwrap();
    createreply.components = Some(components);
    ctx.send(createreply).await?;

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
        modifiers: modifiers_serialized,
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

pub async fn _fish(
    ctx: Context<'_>
) -> Result<(), Error> {
    let custom_data = &ctx.data();
    let loopchans_config = &custom_data.config;

    if rand::rng().random_bool(loopchans_config.economy.fish_fail_chance.into()) {
        ctx.send(CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .description("When you drive to nearest river it turns out your fishing rod is broken 😕\nCome back later 😓")
                    .color(Color::from_rgb(255, 100, 100))
            )
        ).await?;

        return Ok(());
    }

    let (catched_fish, catched_modifiers, catched_size, catched_fishmodifiers) = {
        let fishes = &loopchans_config.economy.fishes;

        let mut total_weight: u32 = 0;
        for fish in fishes {
            total_weight += fish.chance;
        }
    
        let weights: Vec<u32> = fishes.iter().map(|fish| total_weight-fish.chance).collect();
        let dist = WeightedIndex::new(&weights).unwrap();

        let mut rng = rng();
        let index = dist.sample(&mut rng);

        let fish: &crate::Fish = &fishes[index];

        let mut modifiers: Vec<String> = vec![];
        let mut fishmodifiers: Vec<FishModifier> = vec![];

        for modifier in &fish.possible_modifiers {
            let real_modifier = fishmodifier_from_name(modifier, &loopchans_config.economy.fishes_modifiers).unwrap();
            if rand::rng().random_range(..=real_modifier.chance) == 1 {
                fishmodifiers.push(real_modifier);
                modifiers.push(modifier.to_string());
            }
        }

        (fish, modifiers, rand::rng().random_range(fish.possible_size[0]..=fish.possible_size[1]), fishmodifiers)
    };
    let catched_modifiers_serialized: String = json::to_string(&catched_modifiers).unwrap();

    let catch_time: u64 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()+20;

    let reply: poise::ReplyHandle<'_> = ctx.send(CreateReply::default()
        .embed(
            CreateEmbed::default()
                .description(format!("<t:{}:R>", catch_time))
                .color(Color::from_rgb(255, 160, 100))
        )
    ).await?;

    tokio::time::sleep(Duration::from_secs(19)).await; // Assuming sending takes ~1 second

    let mut final_size: f32 = catched_size;
    let mut final_value: f64 = catched_fish.base_value as f64;

    let modifiers: Vec<FishModifier> = catched_fishmodifiers;
    if modifiers.len() > 0 {
        for modifier in modifiers {
            if modifier.size_multiplier.is_some() {
                final_size *= modifier.size_multiplier.unwrap()
            }

            if modifier.value_multiplier.is_some() {
                final_value *= modifier.value_multiplier.unwrap() as f64
            }
        }
    }

    final_size = (final_size*100.0).floor()/100.0;
    final_value = (final_value*final_size as f64).floor();

    let uuid = Uuid::new_v4().to_string();

    let successfully_gave_fish: Result<usize, async_sqlite::Error> = give_fish_to_user_in_fishing_db(&custom_data.db_client, ctx.author().id.get(), DataFish {
        uuid: uuid.clone(),
        modifiers: catched_modifiers_serialized,
        r#type: catched_fish.name.clone(),
        size: catched_size
    }).await;

    if successfully_gave_fish.is_err() {
        error!("Failed to give fish to {}: {}", ctx.author().id.get(), successfully_gave_fish.unwrap_err().to_string());

        reply.edit(ctx, CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .description("Failed to give you fish! Please try again later, if the issue persists contact <@908779319084589067>")
                    .color(Color::from_rgb(255, 100, 100))
            )
        ).await?;

        ctx.set_invocation_data(true).await; // cancel cooldown (hopefully)

        return Ok(());
    }

    reply.edit(ctx, CreateReply::default()
        .embed(
            CreateEmbed::default()
                .description(
                    format!(
                        "You catched **{} {} • {}cm! *(~${})***\n*\"{}\"*\n-# ID: {}\n-# Check your inventory for more information.",
                        catched_modifiers.join(" "), catched_fish.name, final_size, final_value, catched_fish.description, uuid
                    )
                )
                .color(Color::from_rgb(100, 255, 100))
        )
    ).await?;

    Ok(())
}

pub async fn _profish( // TODO: Fishing Minigame
    ctx: Context<'_>
) -> Result<(), Error> {
    ctx.send(CreateReply::default()
        .embed(
            CreateEmbed::default()
                .description("wip")
        )
    ).await?;

    Ok(())
}

/// Catch a fish! (or not...)
#[poise::command(slash_command)]
pub async fn fish(
    ctx: Context<'_>,
    #[description = "Enable catching minigame for better fish"]
    pro_mode: Option<bool>
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

    if pro_mode.is_some() {
        if pro_mode.unwrap() {
            // MINIGAME
            return _profish(ctx).await;
        }
    }

    // Basic waiting fishing
    return _fish(ctx).await;
}