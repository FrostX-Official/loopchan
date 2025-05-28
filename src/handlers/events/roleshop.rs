use serenity::all::{ButtonStyle, Color, ComponentInteraction, CreateActionRow, CreateButton, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage, Guild, RoleId};
use tracing::{error, warn};

use crate::{utils::database::economy::{decrement_user_balance_in_eco_db, get_roleshopitem_by_id, get_user_balance_in_eco_db}, RoleShopItem};

pub async fn handle_roleshop_selector(
    ctx: &serenity::prelude::Context,
    interaction: ComponentInteraction,
    data: &crate::Data
) {
    let selector_option_id: Option<&Vec<std::string::String>> = match &interaction.data.kind {
        serenity::all::ComponentInteractionDataKind::StringSelect { values } => { Some(values) }
        _ => { None }
    };

    if selector_option_id.is_none() {
        return;
    }

    let selector_option_id = selector_option_id.unwrap()[0].clone();
    let pressed_button_role_id_str: String = selector_option_id.clone().split_off(9);
    let pressed_button_role_id: u64 = pressed_button_role_id_str.parse().unwrap();

    let shop_item: Result<&RoleShopItem, std::io::Error> = get_roleshopitem_by_id(pressed_button_role_id, &data.config.economy.shop_items).await;

    if shop_item.is_err() {
        warn!("Not found shop item with ID: {}", selector_option_id);
        return;
    }
    let shop_item: &RoleShopItem = shop_item.unwrap();

    let author_id: u64 = interaction.user.id.get();

    let db_client: &async_sqlite::Client = &data.db_client; // User's data created in eco_db in pre_command hook, so no need to worry about that
    let balance_check: Result<u64, async_sqlite::Error> = get_user_balance_in_eco_db(db_client, author_id).await;

    let components = vec![
        CreateActionRow::Buttons(vec![
            CreateButton::new(format!("roleshop.buy.{}", shop_item.id))
                .label("Buy")
                .style(ButtonStyle::Success),
    ])];
    
    if !balance_check.is_ok() {
        error!("Failed to check {}'s balance: {}", author_id, balance_check.unwrap_err().to_string());
        interaction.create_response(
            ctx,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::default()
                    .embed(
                        CreateEmbed::default()
                            .title(format!("Role :: <:{}:{}> {}", shop_item.icon_name, shop_item.icon_id, shop_item.display_name))
                            .description(format!("*{}*\n\nActual Role: **<@&{}>**\nPrice: **${}**\n-# also failed to check your balance <:LoopchanOopsie:1376849367880826941>", shop_item.description, shop_item.id, shop_item.price))
                            .color(Color::from_rgb(255, 255, 255))
                    )
                    .components(components)
                    .ephemeral(true)
            )
        ).await.unwrap();

        return;
    }

    let balance: u64 = balance_check.unwrap();

    interaction.create_response(
        ctx,
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::default()
                .embed(
                    CreateEmbed::default()
                        .title(format!("Role :: <:{}:{}> {}", shop_item.icon_name, shop_item.icon_id, shop_item.display_name))
                        .description(format!("*{}*\n\nActual Role: **<@&{}>**\nPrice: **${}**\n\n*Your balance: **${}***", shop_item.description, shop_item.id, shop_item.price, balance))
                        .color(Color::from_rgb(255, 255, 255))
                )
                .components(components)
                .ephemeral(true)
        )
    ).await.unwrap();
}

pub async fn handle_roleshop_buy( // TODO: Improve design
    ctx: &serenity::prelude::Context,
    interaction: ComponentInteraction,
    data: &crate::Data
) {
    let role_id_str: String = interaction.data.custom_id.clone().split_off(13);
    let role_id: u64 = role_id_str.parse().unwrap();

    let member = Guild::get(ctx, data.config.guild).await.unwrap().member(ctx, interaction.user.id).await.unwrap();
    if member.roles.contains(&RoleId::new(role_id)) {
        interaction.create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::default()
                    .embed(
                        CreateEmbed::default()
                            .description("You already have this role bought.")
                            .color(Color::from_rgb(255, 100, 100))
                    )
                    .components(vec![])
                    .ephemeral(true)
            )
        ).await.unwrap();
        return;
    }

    let shop_items: &Vec<RoleShopItem> = &data.config.economy.shop_items;
    let shop_item: Result<&RoleShopItem, std::io::Error> = get_roleshopitem_by_id(role_id, shop_items).await;

    if shop_item.is_err() {
        warn!("Not found shop item with ID: {}", role_id);
        return;
    }
    let shop_item: &RoleShopItem = shop_item.unwrap();

    let db_client = &data.db_client;
    let author_id: u64 = interaction.user.id.get();

    let balance_check: Result<u64, async_sqlite::Error> = get_user_balance_in_eco_db(db_client, author_id).await;
    if !balance_check.is_ok() {
        interaction.create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::default()
                    .embed(
                        CreateEmbed::default()
                            .description("Failed to check your balance. Please try again later.")
                            .color(Color::from_rgb(255, 100, 100))
                    )
                    .components(vec![])
                    .ephemeral(true)
            )
        ).await.unwrap();
        return;
    }

    let balance: u64 = balance_check.unwrap();

    if shop_item.price as u64 > balance {
        let components = vec![
            CreateActionRow::Buttons(vec![
                CreateButton::new(format!("roleshop.buy.{}", shop_item.id))
                    .label("Retry")
                    .style(ButtonStyle::Danger),
        ])];

        interaction.create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::default()
                    .embed(
                        CreateEmbed::default()
                            .description("Insufficient Funds")
                            .color(Color::from_rgb(255, 100, 100))
                    )
                    .components(components)
                    .ephemeral(true)
            )
        ).await.unwrap();
        return;
    }

    let successful_decrement = decrement_user_balance_in_eco_db(db_client, author_id, shop_item.price.into()).await;

    if successful_decrement.is_err() {
        error!("Failed to decrease {}'s balance: {}", author_id, successful_decrement.unwrap_err().to_string());
        let components = vec![
            CreateActionRow::Buttons(vec![
                CreateButton::new(format!("roleshop.buy.{}", shop_item.id))
                    .label("Retry")
                    .style(ButtonStyle::Danger),
        ])];

        interaction.create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::default()
                    .embed(
                        CreateEmbed::default()
                            .description("Failed to decrease your balance. Please try again later or contact <@908779319084589067>.")
                            .color(Color::from_rgb(255, 100, 100))
                    )
                    .components(components)
                    .ephemeral(true)
            )
        ).await.unwrap();
        return;
    }

    let successfully_gave_shop_role: Result<(), serenity::Error> = member.add_role(ctx, RoleId::new(shop_item.id)).await;

    if successfully_gave_shop_role.is_err() {
        error!("Failed to give {} role to {}: {}", shop_item.id, author_id, successful_decrement.unwrap_err().to_string());
        interaction.create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::default()
                    .embed(
                        CreateEmbed::default()
                            .description("Failed to give you role. Please contact <@908779319084589067> to receive your funds back.")
                            .color(Color::from_rgb(255, 100, 100))
                    )
                    .components(vec![])
                    .ephemeral(true)
            )
        ).await.unwrap();
        return;
    }

    interaction.create_response(
        ctx,
        CreateInteractionResponse::UpdateMessage(
            CreateInteractionResponseMessage::default()
                .embed(
                    CreateEmbed::default()
                        .description("Successfully bought role!")
                        .color(Color::from_rgb(100, 255, 100))
                )
                .components(vec![])
                .ephemeral(true)
        )
    ).await.unwrap();
}

pub async fn handle_interaction(
    ctx: &serenity::prelude::Context,
    interaction: ComponentInteraction,
    data: &crate::Data
) {
    let interaction_id = &interaction.data.custom_id;
    if interaction_id == "roleshop.selector" {
        return handle_roleshop_selector(ctx, interaction, data).await;
    }
    if interaction_id.starts_with("roleshop.buy.") {
        return handle_roleshop_buy(ctx, interaction, data).await;
    }
}