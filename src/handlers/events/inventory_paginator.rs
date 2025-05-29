use serenity::all::{Color, ComponentInteraction, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage};
use tracing::error;

use crate::{commands::fishing::{get_inventory_components, get_inventory_embeds_after_interaction}, utils::database::fishing::get_user_fishes_in_fishing_db, DataFish};


pub async fn handle_interaction(
    ctx: &serenity::prelude::Context,
    interaction: ComponentInteraction,
    data: &crate::Data
) {
    let interaction_id: &String = &interaction.data.custom_id;
    if !interaction_id.starts_with("fishing.inventory.") {
        return;
    }

    let author_id: u64 = interaction.user.id.get();

    let db_client: &async_sqlite::Client = &data.db_client;
    let inventory: Result<Vec<DataFish>, async_sqlite::Error> = get_user_fishes_in_fishing_db(db_client, author_id).await;

    if inventory.is_err() {
        error!("Failed to get {}'s fishes: {}", author_id, inventory.unwrap_err().to_string());

        interaction.create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::default()
                    .embed(
                        CreateEmbed::default()
                            .description("Failed to find your fishes! Please try again later, if the issue persists contact <@908779319084589067>")
                            .color(Color::from_rgb(255, 100, 100))
                    )
                    .components(vec![])
                    .ephemeral(true)
            )
        ).await.unwrap();

        return;
    }

    let inventory: Vec<DataFish> = inventory.unwrap();
    let inventory_size = inventory.len();

    if inventory_size == 0 {
        interaction.create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::default()
                    .embed(
                        CreateEmbed::default()
                            .title("🎣 Inventory")
                            .description("Your inventory is empty 😥\nYou can catch fish by using command ```/fishing fish```")
                            .color(Color::from_rgb(255, 100, 100))
                    )
                    .components(vec![])
                    .ephemeral(true)
            )
        ).await.unwrap();

        return;
    }

    let mut current_page: u32 = interaction_id.split(".").last().unwrap().parse().unwrap();
    if interaction_id.starts_with("fishing.inventory.prev") {
        current_page -= 1;
    } else if interaction_id.starts_with("fishing.inventory.superprev") {
        current_page = 0;
    } else if interaction_id.starts_with("fishing.inventory.supernext") {
        current_page = inventory_size as u32/5;
    } else {
        current_page += 1;
    }

    let embeds: Option<Vec<CreateEmbed>> = get_inventory_embeds_after_interaction(ctx, &interaction, data, inventory, current_page).await;

    if embeds.is_none() {
        interaction.create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::default()
                    .embed(
                        CreateEmbed::default()
                            .description("Failed to find your fishes! Please try again later, if the issue persists contact <@908779319084589067>")
                            .color(Color::from_rgb(255, 100, 100))
                    )
                    .components(vec![])
                    .ephemeral(true)
            )
        ).await.unwrap();

        return;
    }

    let components = get_inventory_components(current_page, inventory_size as u32);

    let createresponse = CreateInteractionResponseMessage::default()
        .embeds(embeds.unwrap())
        .components(components);

    interaction.create_response(
        ctx,
        CreateInteractionResponse::UpdateMessage(createresponse)
    ).await.unwrap();
}