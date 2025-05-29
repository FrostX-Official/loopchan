use serenity::all::{Color, ComponentInteraction, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage};

pub async fn handle_interaction(
    ctx: &serenity::prelude::Context,
    interaction: ComponentInteraction,
    _data: &crate::Data
) {
    let interaction_id: &String = &interaction.data.custom_id;
    if interaction_id != "fishing.minigame.fish" {
        interaction.create_response(
            ctx,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::default()
                    .embed(
                        CreateEmbed::default()
                            .description("You missed!")
                            .color(Color::from_rgb(255, 100, 100))
                    )
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
                        .title("Processing...")
                        .color(Color::from_rgb(255, 160, 100))
                )
                .components(vec![])
                .ephemeral(true)
        )
    ).await.unwrap();
}