use serenity::all::{ButtonStyle, Colour, CreateActionRow, CreateButton, CreateEmbed};

use crate::utils::basic::{is_qa, parse_env_as_string};
use crate::{Context, Error};

/// QA Managing Commands
#[poise::command(slash_command, subcommands("sendform"), subcommand_required)]
pub async fn qa(_ctx: Context<'_>) -> Result<(), Error> { Ok(()) }

// This command has useful Option<User> handler (choose yourself incase you don't choose anyone), so it will be commented out for now.
// Checks if user in QA program.
// The most useless command yet (just check if member has QA role)
//
// #[poise::command(slash_command)]
// pub async fn status(
//     ctx: Context<'_>,
//     #[description = "User"] user: Option<serenity::model::user::User>
// ) -> Result<(), Error> {
//     let custom_data = ctx.data();
//
//     let nuser: serenity::model::user::User;
//
//     if user.is_none() {
//         nuser = ctx.author().clone();
//     } else {
//         nuser = user.unwrap();
//     }
//
//     let is_qa_v: bool = is_qa(ctx, &nuser).await;
//     db::create_user_in_db(&custom_data.db_client, nuser.id.into(), 0).await?;
//
//     ctx.send(poise::CreateReply::default()
//         .content(is_qa_v.to_string())
//         .ephemeral(true)
//     ).await?;
//
//     Ok(())
// }

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

    user.direct_message(ctx, serenity::all::CreateMessage::default()
        .embed(
            CreateEmbed::default()
                .title("QA Team Invitation")
                .description(
                    "Hello! You have been chosen to participate in closed **PARKOUR: The Loop** testing.\nSince you're a trusted member of our community we are sending you a link to QA form!\n\n***".to_owned()
                        +&parse_env_as_string("QA_FORM_LINK")
                        +"***\n\n*Please note that leaking this link is not allowed and will result in removing your testing access or (if you don't have one) permament ban from PTL!*\n-# Enjoy :D"
                )
                .color(Colour::from_rgb(255, 255, 255))
        ).components(components)
    ).await?;
    Ok(())
}