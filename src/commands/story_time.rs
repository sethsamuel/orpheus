use std::collections::HashSet;

use chrono::Utc;

use crate::{
    discord::thread,
    telephone::{
        consts::{FINISHED, STORY_TELLER},
        Telephone,
    },
    types::{Context, DiscordMessage, Error},
};

#[poise::command(slash_command)]
pub async fn story_time(
    ctx: Context<'_>,
    #[description = "Shared folder"] folder_url: String,
) -> Result<(), Error> {
    _ = ctx.defer().await;

    let telephone = Telephone {
        host: ctx.author().id,
        folder_url,
        lead: None,
        players: vec![],
        finished_players: HashSet::new(),
        nag_interval: 7,
        nagged_at: Utc::now().into(),
    };

    let (channel, message_id) = thread::create(
        ctx.http(),
        ctx.channel_id(),
        "Narrative telephone",
        telephone,
    )
    .await;
    ctx.http()
        .create_reaction(
            channel.id,
            message_id,
            &serenity::all::ReactionType::Unicode(STORY_TELLER.to_string()),
        )
        .await
        .inspect_err(|e| println!("Failed to add emoji! {:?}", e))?;

    ctx.http()
        .create_reaction(
            channel.id,
            message_id,
            &serenity::all::ReactionType::Unicode(FINISHED.to_string()),
        )
        .await
        .inspect_err(|e| println!("Failed to add emoji! {:?}", e))?;

    ctx.http()
        .create_reaction(
            channel.id,
            message_id,
            &serenity::all::ReactionType::Unicode(FINISHED.to_string()),
        )
        .await
        .inspect_err(|e| println!("Failed to add emoji! {:?}", e))?;

    _ = ctx.data().tx.send(DiscordMessage {
        channel_id: channel.id,
        message_id,
    });

    _ = ctx
        .say(format!("Created a new thread <#{}>", channel.id))
        .await;

    Ok(())
}
