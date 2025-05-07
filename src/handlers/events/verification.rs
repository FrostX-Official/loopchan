use serenity::all::{ButtonStyle, Colour, ComponentInteraction, CreateActionRow, CreateButton, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage, EditInteractionResponse, Guild, RoleId};

use crate::utils::{basic::remove_whitespace, database::linking::update_roblox_id_in_users_db};

pub async fn handle_interaction(
    ctx: &serenity::prelude::Context,
    interaction: ComponentInteraction,
    data: &crate::Data
) {
    let pressed_button_id = &interaction.data.custom_id;
    if !pressed_button_id.starts_with("verification") {
        return;
    }

    let roblox_client = &data.roblox_client;

    let no_whitespace_wordgen: String;
    let roblox_user_id: u64;
    {
        let verifications_data_lock = data.verifications.lock().await;
        let (wordgen, id) = verifications_data_lock.get(&interaction.user.id.get()).unwrap();
        no_whitespace_wordgen = wordgen.clone(); // hpfully not expensive
        roblox_user_id = id.clone();
    }

    interaction.create_response(
        ctx,
        CreateInteractionResponse::UpdateMessage(
            CreateInteractionResponseMessage::default()
                .components(vec![])
                .embeds(vec![])
                .content("Processing... Please wait.")
        )
    ).await.unwrap();

    if pressed_button_id == "verification.cancel" {
        // Cancel verification
        interaction.edit_response(
            ctx,
            EditInteractionResponse::default()
                .content("❌ Verification Cancelled.")
        )
        .await.unwrap();

        return;
    }

    if pressed_button_id == "verification.regenerate" {
        // Regenerate wordgen
        let components: CreateActionRow = CreateActionRow::Buttons(vec![
            CreateButton::new("verification.check")
                .label("Done")
                .style(ButtonStyle::Primary)
                .emoji('✅'),
            CreateButton::new("verification.cancel")
                .label("Cancel")
                .style(ButtonStyle::Secondary)
                .emoji('❌'),
            CreateButton::new("verification.regenerate")
                .label("Regenerate")
                .style(ButtonStyle::Secondary)
                .emoji('🔃')
                .disabled(true) // TODO: Maybe make regeneration always enabled, but add cooldown to it .. ?
        ]);

        let mut randomwords: Vec<String> = vec![];
        for _ in 0..11 {
            randomwords.insert(0, crate::utils::wordgen::getrandomgenword().await);
        }

        let builder = EditInteractionResponse::default()
            .content("")
            .add_embed(
                CreateEmbed::default()
                    .title("Found User!")
                    .description(
                        format!("**Please confirm that this is your Roblox Account by changing your profile description to:**\n```{}```\n## You have 5 minutes.\n-# You can change it back after verification process! (Make sure to save it though :D)", randomwords.join("\n"))
                    )
                    .color(Colour::from_rgb(255, 255, 100))
            )
            .components(vec![components]);

        randomwords.remove(0); // For some reason if you join vector with \n separator it will not show first element in embed. This is why we're deleting it after creating embed
        let no_whitespace_wordgen = remove_whitespace(&randomwords.join("\n"));

        data.verifications.lock().await.insert(interaction.user.id.get(), (no_whitespace_wordgen, roblox_user_id));

        interaction.edit_response(
            ctx,
            builder)
        .await.unwrap();

        return;
    }

    // Check if wordgens match
    let user_details_fetch: Result<roboat::users::UserDetails, roboat::RoboatError> = roblox_client.user_details(roblox_user_id).await;
    if !user_details_fetch.is_ok() {
        interaction.edit_response(
            ctx,
            EditInteractionResponse::default()
                .content("Failed to verify your account!\nPlease try again later.")
        )
        .await.unwrap();
        return;
    }

    let user_details_fetch_unwrapped: roboat::users::UserDetails = user_details_fetch.unwrap();
    let user_description: String = user_details_fetch_unwrapped.description;
    let no_whitespace_description = remove_whitespace(&user_description);
    
    if no_whitespace_description != no_whitespace_wordgen {
        interaction.edit_response(
            ctx,
            EditInteractionResponse::default()
                .content("Your Roblox profile description does not match wordgen.\nIf you think that's not true contact <@908779319084589067> for support!\nYou can try again later.")
        )
        .await.unwrap();
        return;
    }

    // Change user's roblox_id in db to new, verified one

    let successfully_updated_data: Result<usize, async_sqlite::Error> = update_roblox_id_in_users_db(&data.db_client, interaction.user.id.get(), roblox_user_id).await;
    if !successfully_updated_data.is_ok() {
        eprintln!("{}", &successfully_updated_data.err().unwrap().to_string());
        interaction.edit_response(
                ctx,
                EditInteractionResponse::default()
                .content("Failed to verify your account!\nPlease try again later or report this issue to <@908779319084589067>!")
        )
        .await.unwrap();
        return;
    }

    // TODO: Also update roles depending on data in game

    let member = Guild::get(ctx, data.config.guild).await.unwrap().member(ctx, interaction.user.id).await.unwrap();
    let successfully_gave_member_role: Result<(), serenity::Error> = member.add_role(ctx, RoleId::new(data.config.roles.member)).await;
    if !successfully_gave_member_role.is_ok() {
        eprintln!("{}", &successfully_gave_member_role.err().unwrap().to_string());
        interaction.edit_response(
            ctx,
            EditInteractionResponse::default()
                .content("") // Clear text, leave only embed
                .embed(
                    CreateEmbed::default()
                        .title("Verified Account!")
                        .description(
                            "Thank you for verification!\nOnce the game comes out you will be able to update your roles, depending on your data ingame :D\n-# Failed to give out member role though! Please contact <@&1334231212851466311> for that."
                        )
                        .color(Colour::from_rgb(80, 255, 80))
                )
        )
        .await.unwrap();
    } else {
        interaction.edit_response(
            ctx,
            EditInteractionResponse::default()
                .content("") // Clear text, leave only embed
                .embed(
                    CreateEmbed::default()
                        .title("Verified Account!")
                        .description(
                            "Thank you for verification!\nOnce the game comes out you will be able to update your roles, depending on your data ingame :D"
                        )
                        .color(Colour::from_rgb(80, 255, 80))
                )
        ).await.unwrap();
    }
}