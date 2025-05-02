use serenity::all::{ButtonStyle, Color, CreateActionRow, CreateButton, CreateEmbed};

use crate::utils::basic::{is_qa, parse_env_as_string};
use crate::{Context, Error};

/// QA Managing Commands
#[poise::command(slash_command, subcommands("sendform"), subcommand_required)]
pub async fn qa(_ctx: Context<'_>) -> Result<(), Error> { Ok(()) }

/// Send QA form to specific member
#[poise::command(slash_command)]
pub async fn sendform(
    ctx: Context<'_>,
    #[description = "Member"] user: serenity::model::user::User
) -> Result<(), Error> {
    // Check if user is already QA
    if is_qa(ctx, &user).await {
        ctx.send(poise::CreateReply::default()
            .content("This user is already in QA program.")
            .ephemeral(true)
        ).await?;
        return Ok(());
    }

    // These are handled in [main.rs] -> [handle_message_component]
    let components: Vec<CreateActionRow> = vec![ // Buttons
        CreateActionRow::Buttons(vec![
            CreateButton::new("qa.invitation.accept")
                .label("Accept")
                .style(ButtonStyle::Primary)
                .emoji('✅'),
            CreateButton::new("qa.invitation.deny")
                .label("Deny")
                .style(ButtonStyle::Primary)
                .emoji('❌'),
        ]),
        CreateActionRow::Buttons(vec![ // Buttons Descriptions
            CreateButton::new("qa.invitation.accept.description")
                .label("Accept once form is filled out.")
                .style(ButtonStyle::Secondary)
                .emoji('✅').disabled(true),
            CreateButton::new("qa.invitation.deny.description")
                .label("Deny whenever you want.")
                .style(ButtonStyle::Secondary)
                .emoji('❌').disabled(true),
        ]),
    ];

    let sent_message: Result<serenity::model::prelude::Message, serenity::Error> = user.direct_message(ctx, serenity::all::CreateMessage::default()
        .embed(
            CreateEmbed::default()
                .title("QA Team Invitation")
                .description(
                    "Hello! You have been chosen to participate in closed **PARKOUR: The Loop** testing.\nSince you're a trusted member of our community we are sending you a link to QA form!\n\n***".to_owned()
                        +&parse_env_as_string("QA_FORM_LINK")
                        +"***\n\n*Please note that leaking this link is not allowed and will result in removing your testing access or (if you don't have one) permament ban from PTL!*\n-# Enjoy :D"
                )
                .color(Color::from_rgb(255, 255, 255))
        ).components(components)
    ).await;

    if sent_message.is_ok() {
        ctx.send(poise::CreateReply::default()
            .content("Successfully sent user QA Form link.")
            .ephemeral(true)
        ).await?;
    } else {
        ctx.send(poise::CreateReply::default()
            .content("Failed to send QA form link to user!\n-# ".to_owned()+&sent_message.err().unwrap().to_string())
            .ephemeral(true)
        ).await?;
    }

    Ok(())
}