use tokio::io::AsyncWriteExt;

use ab_glyph::{FontRef, PxScale, VariableFont};

use ::serenity::all::{CreateAttachment, ChannelId, Color, CreateEmbed, CreateMessage, EmojiId, Member};

use poise::serenity_prelude as serenity;

use tracing::{warn, error};

use image::{DynamicImage, Rgb, Rgba};
use image::imageops::{overlay, resize, FilterType};
use imageproc::drawing::draw_text_mut;
use std::path::Path;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub async fn welcomecard(
    ctx: &serenity::Context,
    new_member: &Member,
    data: &crate::Data
) -> Result<(), Error> {
    // warn!("{} joined ptl.", &new_member.user.name);
    let loopchans_config = &data.config;
    if !loopchans_config.welcomecard.enabled { return Ok(()) }
    let ptl_channels: std::collections::HashMap<ChannelId, serenity::model::prelude::GuildChannel> = ctx.cache.guild(loopchans_config.guild).unwrap().channels.clone();
    let welcomes_channel = ptl_channels.get(&loopchans_config.welcomecard.channel.unwrap().into());
    if welcomes_channel.is_none() {
        warn!("Failed to find welcomes channel to welcome member in!");
        return Ok(());
    }

    let member_count: u64 = ctx.cache.guild(loopchans_config.guild).unwrap().member_count;

    let mut image: image::ImageBuffer<Rgb<u8>, Vec<u8>> = image::open(Path::new("welcomecardtemplate.png")).unwrap().into();

    let mut font: FontRef<'_> = FontRef::try_from_slice(include_bytes!("../../../CascadiaCode.ttf")).unwrap();
    font.set_variation(b"lght", 400.0);

    let scale: PxScale = PxScale { x: 74.0, y: 74.0, };
    let (member_username, member_userid) = (&new_member.user.name, &new_member.user.id.get());

    draw_text_mut(&mut image, Rgb([255, 255, 255]), 79, 582, scale, &font, member_username);
    draw_text_mut(&mut image, Rgb([255, 255, 255]), 79, 656, scale, &font, &format!("you’re member #{}!", member_count));
    
    // ^ If I make image initially Rgba draw_text_mut just breaks for some reason (even if I provide Rgba to color)
    // So we convert it to Rgba buffer after adding text:
    let mut image: image::ImageBuffer<Rgba<u8>, Vec<u8>> = DynamicImage::from(image).into();

    let avatar_url = new_member.user.static_avatar_url();
    if avatar_url.is_some() {
        let avatar_url: String = avatar_url.unwrap();
        let response: Result<roboat::reqwest::Response, roboat::reqwest::Error> = roboat::reqwest::get(&avatar_url).await;

        if response.is_ok() { // nesting goesw hard
            let response: roboat::reqwest::Response = response.unwrap();
            let is_cool = response.status().is_success();
            if is_cool {
                // Download file into temp
                // https://cdn.discordapp.com/avatars/exampleid/examplehash.png?size=1024 >>> examplehash.png?size=1024 >>> examplehash.png
                let avatar_file_name = avatar_url.split("/").last().unwrap().split("?").next().unwrap();
                let avatar_path = format!("temp/{}", avatar_file_name);
                let mut file = tokio::fs::File::create(&avatar_path).await?;
                file.write_all(&response.bytes().await.unwrap().to_vec().as_slice()).await?;

                // Prepare and resize image
                let pfp_image: image::ImageBuffer<Rgba<u8>, Vec<u8>> = image::open(avatar_path).unwrap().into();
                let mut pfp_image: image::ImageBuffer<Rgba<u8>, Vec<u8>> = resize(&pfp_image, 256, 256, FilterType::Nearest);
                
                // Make pixels outside of circle radius (128) completely transparent :P
                for x in 0..=255 {
                    for y in 0..=255 {
                        if f64::hypot((x as f64) - 128.0, (y as f64) - 128.0) > 128.5 {
                            pfp_image.put_pixel(x, y, Rgba([0, 0, 0, 1]));
                            pfp_image.put_pixel(y, x, Rgba([0, 0, 0, 1]));
                        }
                    }
                }

                // Overlay pfp ontop of welcome image
                overlay(&mut image, &pfp_image, 1690, 95);
            }
        }
    }

    // warn!("generated {}'s welcome card", &new_member.user.name);

    let mut bytes: Vec<u8> = Vec::new();
    image.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)?;

    // warn!("saved {}'s welcome card", &new_member.user.name);

    let attachment: CreateAttachment = CreateAttachment::bytes(bytes, format!("welcomecard{}.gif", member_userid));
    let filename = attachment.filename.clone();

    let welcome_message: Result<serenity::model::prelude::Message, serenity::Error> = welcomes_channel.unwrap().send_message(ctx,
        CreateMessage::default()
            .add_file(attachment)
            .embed(
                CreateEmbed::default()
                    .description(format!("Welcome <@{}>! Hope you'll enjoy your stay.", member_userid))
                    .attachment(filename)
                    .color(Color::from_rgb(255, 255, 255))
            )
    ).await;

    if welcome_message.is_ok() {
        // warn!("sent {}'s welcome card", &new_member.user.name);
        if !loopchans_config.welcomecard.react.unwrap() {
            return Ok(());
        }
        let welcome_react: serenity::model::prelude::ReactionType = serenity::ReactionType::Custom {
            animated: loopchans_config.welcomecard.react_animated.unwrap(),
            id: EmojiId::new(loopchans_config.welcomecard.react_id.unwrap()),
            name: loopchans_config.welcomecard.react_name.clone()
        };
        welcome_message.unwrap().react(ctx, welcome_react).await?;
    } else {
        error!("Failed to send {}'s welcome card: {}", &new_member.user.name, welcome_message.unwrap_err().to_string());
    }

    Ok(())
}