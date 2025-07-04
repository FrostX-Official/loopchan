// TODO: Trading Fish
// TODO: Throwing fish away for exp

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures::{Stream, StreamExt};
use poise::{CooldownConfig, CreateReply};
use rand::{distr::{weighted::WeightedIndex, Distribution}, rng, Rng};
use serenity::{all::{ButtonStyle, Color, ComponentInteraction, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage}, json};
use tracing::{error, warn};
use uuid::Uuid;

use crate::{utils::{basic::{fish_from_name, fishmodifier_from_name, fishmodifiers_from_datafishmodifiers, get_fishes_names_from_fishes, remove_whitespace}, database::{economy::get_user_level_in_eco_db, fishing::{get_user_fishes_in_fishing_db, give_fish_to_user_in_fishing_db}}}, Context, DataFish, Error, FishModifier};

use super::eco::{exp_needed_to_next_level, give_user_eco_exp};

/// Fishing Commands
#[poise::command(slash_command, subcommands("give_fish", "inventory", "fish", "throwaway"), subcommand_required)]
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
                    CreateButton::new(format!("fishing.inventory.superprev.{current_page}"))
                        .label("⏮")
                        .style(ButtonStyle::Secondary)
                );
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
                buttons.push(
                    CreateButton::new(format!("fishing.inventory.supernext.{current_page}"))
                        .label("⏭")
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

pub async fn get_inventory_embeds_after_interaction(
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

        let actual_fish = fish_from_name(&fish.r#type, &data.config.economy.fishes).unwrap();

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

        let actual_fish = fish_from_name(&fish.r#type, &custom_data.config.economy.fishes).unwrap();

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
#[poise::command(slash_command)]
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
    let fish_names: Vec<String> = get_fishes_names_from_fishes(&ctx.data().config.economy.fishes); // TODO: Cache this in data to make autocomplete faster

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
    let actual_fish: Result<crate::Fish, std::io::Error> = fish_from_name(&r#type, &ctx.data().config.economy.fishes);
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

    let author_id: u64 = ctx.author().id.get();

    if rand::rng().random_bool(loopchans_config.economy.fish_fail_chance.into()) {
        let users_lvl = get_user_level_in_eco_db(&custom_data.db_client, author_id).await;
        if users_lvl.is_err() {
            error!("Failed to check {}'s level: {}", author_id, users_lvl.unwrap_err().to_string());
    
            ctx.send(CreateReply::default()
                .embed(
                    CreateEmbed::default()
                        .description("Failed to check your level! Please try again later, if the issue persists contact <@908779319084589067>")
                        .color(Color::from_rgb(255, 100, 100))
                )
            ).await?;
    
            ctx.set_invocation_data(true).await; // cancel cooldown (hopefully)
    
            return Ok(());
        }
        let users_lvl: u64 = users_lvl.unwrap();
        let exp_needed: u64 = exp_needed_to_next_level(users_lvl);
        let exp_to_give: u64 = rand::rng().random_range(exp_needed/2..exp_needed);

        let successfully_gave_exp: bool = give_user_eco_exp(custom_data, ctx.author(), exp_to_give).await;

        if !successfully_gave_exp {
            ctx.send(CreateReply::default()
                .embed(
                    CreateEmbed::default()
                        .description("Failed to give you experience! Please try again later, if the issue persists contact <@908779319084589067>")
                        .color(Color::from_rgb(255, 100, 100))
                )
            ).await?;

            ctx.set_invocation_data(true).await; // cancel cooldown (hopefully)

            return Ok(());
        }

        ctx.send(CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .description(format!(
                        "When you drive to nearest river it turns out your fishing rod is broken <:LoopchanOhno:1386683400848670800>\nCome back later <:LoopchanSadKitty:1386683506268176545>\n**+{} EXP**", exp_to_give
                    ))
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
            let real_modifier: FishModifier = fishmodifier_from_name(modifier, &loopchans_config.economy.fishes_modifiers).unwrap();
            if real_modifier.incompatible_with.is_some() {
                for modifier in &real_modifier.incompatible_with.clone().unwrap() {
                    if modifiers.contains(modifier) {
                        break
                    }
                }
            }
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

    let successfully_gave_fish: Result<usize, async_sqlite::Error> = give_fish_to_user_in_fishing_db(&custom_data.db_client, author_id, DataFish {
        uuid: uuid.clone(),
        modifiers: catched_modifiers_serialized,
        r#type: catched_fish.name.clone(),
        size: catched_size
    }).await;

    if successfully_gave_fish.is_err() {
        error!("Failed to give fish to {}: {}", author_id, successfully_gave_fish.unwrap_err().to_string());

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

    // TODO: Exp for fishing

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

fn empty_fishing_minigame_matrix() -> Vec<CreateActionRow> {
    vec![
        CreateActionRow::Buttons(
            vec![
                CreateButton::new("fishing.minigame.1_1").style(ButtonStyle::Primary).label("ㅤ"),
                CreateButton::new("fishing.minigame.1_2").style(ButtonStyle::Primary).label("ㅤ"),
                CreateButton::new("fishing.minigame.1_3").style(ButtonStyle::Primary).label("ㅤ"),
                CreateButton::new("fishing.minigame.1_4").style(ButtonStyle::Primary).label("ㅤ"),
                CreateButton::new("fishing.minigame.1_5").style(ButtonStyle::Primary).label("ㅤ")
            ]
        ),
        CreateActionRow::Buttons(
            vec![
                CreateButton::new("fishing.minigame.2_1").style(ButtonStyle::Primary).label("ㅤ"),
                CreateButton::new("fishing.minigame.2_2").style(ButtonStyle::Primary).label("ㅤ"),
                CreateButton::new("fishing.minigame.2_3").style(ButtonStyle::Primary).label("ㅤ"),
                CreateButton::new("fishing.minigame.2_4").style(ButtonStyle::Primary).label("ㅤ"),
                CreateButton::new("fishing.minigame.2_5").style(ButtonStyle::Primary).label("ㅤ")
            ]
        ),
        CreateActionRow::Buttons(
            vec![
                CreateButton::new("fishing.minigame.3_1").style(ButtonStyle::Primary).label("ㅤ"),
                CreateButton::new("fishing.minigame.3_2").style(ButtonStyle::Primary).label("ㅤ"),
                CreateButton::new("fishing.minigame.3_3").style(ButtonStyle::Primary).label("ㅤ"),
                CreateButton::new("fishing.minigame.3_4").style(ButtonStyle::Primary).label("ㅤ"),
                CreateButton::new("fishing.minigame.3_5").style(ButtonStyle::Primary).label("ㅤ")
            ]
        ),
        CreateActionRow::Buttons(
            vec![
                CreateButton::new("fishing.minigame.4_1").style(ButtonStyle::Primary).label("ㅤ"),
                CreateButton::new("fishing.minigame.4_2").style(ButtonStyle::Primary).label("ㅤ"),
                CreateButton::new("fishing.minigame.4_3").style(ButtonStyle::Primary).label("ㅤ"),
                CreateButton::new("fishing.minigame.4_4").style(ButtonStyle::Primary).label("ㅤ"),
                CreateButton::new("fishing.minigame.4_5").style(ButtonStyle::Primary).label("ㅤ")
            ]
        ),
        CreateActionRow::Buttons(
            vec![
                CreateButton::new("fishing.minigame.5_1").style(ButtonStyle::Primary).label("ㅤ"),
                CreateButton::new("fishing.minigame.5_2").style(ButtonStyle::Primary).label("ㅤ"),
                CreateButton::new("fishing.minigame.5_3").style(ButtonStyle::Primary).label("ㅤ"),
                CreateButton::new("fishing.minigame.5_4").style(ButtonStyle::Primary).label("ㅤ"),
                CreateButton::new("fishing.minigame.5_5").style(ButtonStyle::Primary).label("ㅤ")
            ]
        ),
    ]
}

pub async fn _fishminigame(
    ctx: Context<'_>
) -> Result<(), Error> {
    let custom_data = &ctx.data();
    let loopchans_config = &custom_data.config;

    let mut components = empty_fishing_minigame_matrix();

    let row: usize = rand::rng().random_range(0..=4);
    let column: usize = rand::rng().random_range(0..=4);

    match &components[row] {
        CreateActionRow::Buttons(current_buttons) => {
            let mut buttons = current_buttons.clone();
            buttons[column] = 
                CreateButton::new("fishing.minigame.fish")
                    .label("🐟")
                    .style(ButtonStyle::Primary);

            components[row] = CreateActionRow::Buttons(buttons);
        },
        _ => {}
    }

    let (catched_fish, catched_modifiers, catched_size, catched_fishmodifiers) = {
        let fishes = &loopchans_config.economy.fishes;

        let mut highest_chance: u32 = 0;

        let mut total_weight: u32 = 0;
        for fish in fishes {
            if fish.chance > highest_chance {
                highest_chance = fish.chance;
            }
            total_weight += fish.chance;
        }

        total_weight/=2; // since minigame, double the chance of cool fish
        total_weight = total_weight.max(highest_chance);
    
        let weights: Vec<u32> = fishes.iter().map(|fish| total_weight-fish.chance).collect();
        let dist = WeightedIndex::new(&weights).unwrap();

        let mut rng = rng();
        let index = dist.sample(&mut rng);

        let fish: &crate::Fish = &fishes[index];

        let mut modifiers: Vec<String> = vec![];
        let mut fishmodifiers: Vec<FishModifier> = vec![];

        for modifier in &fish.possible_modifiers {
            let real_modifier = fishmodifier_from_name(modifier, &loopchans_config.economy.fishes_modifiers).unwrap();
            if rand::rng().random_range(..=(real_modifier.chance/2)) == 1 { // double chance since minigame
                fishmodifiers.push(real_modifier);
                modifiers.push(modifier.to_string());
            }
        }

        (fish, modifiers, rand::rng().random_range(fish.possible_size[0]..=fish.possible_size[1]), fishmodifiers)
    };
    let catched_modifiers_serialized: String = json::to_string(&catched_modifiers).unwrap();

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

    let mut score: u8 = 0;
    let score_needed: u8 = (final_value/10.0).ceil().min(10.0) as u8;

    let reply = ctx.send(CreateReply::default()
        .embed(
            CreateEmbed::default()
                .description(format!("=0/{}==============================================\nClick the **fish**", score_needed))
        )
        .components(components)
        .ephemeral(true)
    ).await?;

    tokio::time::sleep(Duration::from_millis(500)).await;

    loop {
        if score >= score_needed {
            break;
        }

        'hopping: loop {
            let mut components: Vec<CreateActionRow> = empty_fishing_minigame_matrix();
    
            let row: usize = rand::rng().random_range(0..=4);
            let column: usize = rand::rng().random_range(0..=4);
    
            match &components[row] {
                CreateActionRow::Buttons(current_buttons) => {
                    let mut buttons = current_buttons.clone();
                    buttons[column] = 
                        CreateButton::new("fishing.minigame.fish")
                            .label("🐟")
                            .style(ButtonStyle::Primary);
    
                    components[row] = CreateActionRow::Buttons(buttons);
                },
                _ => {}
            }
    
            reply.edit(ctx, CreateReply::default()
                .embed(
                    CreateEmbed::default()
                        .description(format!("={}/{}=============================================\nClick the **fish**", score, score_needed))
                )
                .components(components)
            ).await?;

            tokio::time::sleep(Duration::from_millis(750)).await;

            let colour = reply.message().await.unwrap().embeds[0].colour;
            if colour.is_none() {
                continue;
            }
            let green: u8 = colour.unwrap().g();
            if green == 160 {
                score += 1;
                break 'hopping;
            } else if green == 159 {
                score = 0;
                break 'hopping;
            }
        }
    }

    let uuid: String = Uuid::new_v4().to_string();

    let author_id = ctx.author().id.get();

    let successfully_gave_fish: Result<usize, async_sqlite::Error> = give_fish_to_user_in_fishing_db(&custom_data.db_client, author_id, DataFish {
        uuid: uuid.clone(),
        modifiers: catched_modifiers_serialized,
        r#type: catched_fish.name.clone(),
        size: catched_size
    }).await;

    if successfully_gave_fish.is_err() {
        error!("Failed to give fish to {}: {}", author_id, successfully_gave_fish.unwrap_err().to_string());

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

    // TODO: Exp for fishing

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

/// Throw away a fish from your inventory! Let it be free! Freedom!!!~
#[poise::command(slash_command)] // TODO: Work on it
pub async fn throwaway(
    ctx: Context<'_>,
    #[description = "A unique ID of fish you want to throw away"]
    fishid: String
) -> Result<(), Error> {
    ctx.reply(format!("test {}", fishid)).await?;

    Ok(())
}

/// Catch a fish! (or not...)
#[poise::command(slash_command)]
pub async fn fish(
    ctx: Context<'_>,
    #[description = "Enable catching minigame for better fish"]
    minigame: Option<bool>
) -> Result<(), Error> {
    let custom_data: &crate::Data = &ctx.data();
    let economy_config: &crate::EconomyConfig = &custom_data.config.economy;
    let on_cooldown: i32;

    {
        let mut cooldown_tracker = ctx.command().cooldowns.lock().unwrap();

        let mut cooldown_durations: CooldownConfig = CooldownConfig::default();
        cooldown_durations.user = Some(Duration::from_secs(economy_config.fish_cooldown*60));

        if minigame.is_some() {
            if minigame.unwrap() {
                cooldown_durations.user = Some(Duration::from_secs(economy_config.fish_cooldown_mg*60));
            }
        }

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

    if minigame.is_some() {
        if minigame.unwrap() {
            // MINIGAME
            return _fishminigame(ctx).await;
        }
    }

    // Basic waiting fishing
    return _fish(ctx).await;
}