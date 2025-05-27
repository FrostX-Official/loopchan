// TODO: Add roleshop.buy.{id} handler

use serenity::all::{ButtonStyle, Color, ComponentInteraction, CreateActionRow, CreateButton, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage};
use tracing::{error, warn};

use crate::{utils::database::economy::get_user_balance_in_eco_db, RoleShopItem};

pub async fn handle_interaction(
    ctx: &serenity::prelude::Context,
    interaction: ComponentInteraction,
    data: &crate::Data
) {
    let interaction_id = &interaction.data.custom_id;
    if interaction_id != "roleshop.selector" {
        return;
    }
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

    let shop_items: &Vec<toml::Value> = &data.config.economy.shop_items;
    let mut shop_item: Option<RoleShopItem> = None;
    for item in shop_items {
        let item_unwrapped: &toml::map::Map<String, toml::Value> = item.as_table().unwrap();
        let item_prepared: RoleShopItem = RoleShopItem { // I hate this, what the actual fuck is this?? .unwrap().unwrap().unwrap().unwrap().unwrap().unwrap().unwrap().unwrap().unwrap().unwrap().unwrap().unwrap().unwrap().unwrap() 🤖
            id: item_unwrapped.get("id").unwrap().as_integer().unwrap() as u64,
            icon_id: item_unwrapped.get("icon_id").unwrap().as_integer().unwrap() as u64,
            icon_name: item_unwrapped.get("icon_name").unwrap().as_str().unwrap().to_string(),
            display_name: item_unwrapped.get("display_name").unwrap().as_str().unwrap().to_string(),
            description: item_unwrapped.get("description").unwrap().as_str().unwrap().to_string(),
            price: item_unwrapped.get("price").unwrap().as_integer().unwrap() as u32,
        };

        if item_prepared.id == pressed_button_role_id {
            shop_item = Some(item_prepared);
        }
    }

    if shop_item.is_none() {
        warn!("Not found shop item with ID: {}", selector_option_id);
        return;
    }
    let shop_item: RoleShopItem = shop_item.unwrap();

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