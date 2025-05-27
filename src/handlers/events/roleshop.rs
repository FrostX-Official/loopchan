use serenity::all::{ComponentInteraction, CreateInteractionResponse, CreateInteractionResponseMessage};

pub async fn handle_interaction(
    ctx: &serenity::prelude::Context,
    interaction: ComponentInteraction,
    _data: &crate::Data
) {
    let pressed_button_id = &interaction.data.custom_id;
    if !pressed_button_id.starts_with("roleshop") {
        return;
    }

    interaction.create_response(
        ctx,
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::default()
                .content("indevvv :3")
                .ephemeral(true)
        )
    ).await.unwrap();
}