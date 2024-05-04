use ::serenity::all::CacheHttp;
use ::serenity::all::Reaction;
use ::serenity::all::ReactionType;
use ::serenity::all::UserId;
use poise::serenity_prelude as serenity;
use std::collections::HashMap;

use crate::poll::consts::NumberEmojis;
use crate::poll::consts::FINISHED;
use crate::poll::consts::NUMBERS;
use crate::poll::Poll;

use super::types::{Error, State};
use serenity::all::Message;
use serenity::all::Ready;

pub fn on_ready(data_about_bot: &Ready) -> Result<(), Error> {
    println!("Logged in as {}", data_about_bot.user.name);
    Ok(())
}

pub async fn on_message(
    _message: &Message,
    _ctx: &serenity::Context,
    _event: &serenity::FullEvent,
    _data: &State,
) -> Result<(), Error> {
    Ok(())
}

pub async fn on_reaction_change(
    reaction: &Reaction,
    ctx: &serenity::Context,
    _: &serenity::FullEvent,
    data: &State,
) -> Result<(), Error> {
    let bot_id = ctx.http().get_current_user().await.unwrap().id;
    if reaction.user_id.unwrap() == bot_id {
        return Ok(());
    }

    let _lock = data.lock.lock();
    let message = reaction.message(ctx.http()).await.unwrap();
    let poll = Poll::try_from(message.content.clone()).unwrap();
    println!("{:?}", poll);
    println!(
        "New reaction by {} {} to {}",
        reaction.user_id.unwrap(),
        reaction.emoji,
        reaction.message_id
    );

    let complete_users = message
        .reaction_users(
            ctx.http(),
            ReactionType::Unicode(FINISHED.to_string()),
            None,
            None,
        )
        .await
        .unwrap()
        .into_iter()
        .map(|u| u.id)
        .filter(|id| *id != bot_id)
        .collect::<Vec<UserId>>();
    let mut users_map: HashMap<&NumberEmojis, Vec<UserId>> = HashMap::new();

    for n in NUMBERS.iter() {
        let reaction_type = Box::pin(ReactionType::Unicode(n.as_str().to_string()));

        let f =
            ctx.http()
                .get_reaction_users(message.channel_id, message.id, &reaction_type, 50, None);
        let users = f.await.unwrap();
        users_map.insert(
            n,
            users
                .into_iter()
                .map(|u| u.id)
                .filter(|id| complete_users.contains(id))
                .collect(),
        );
    }

    let mut day_counts: HashMap<&NumberEmojis, usize> = HashMap::new();

    for n in NUMBERS.iter() {
        day_counts.insert(n, 0);
        users_map.get(&n).unwrap_or(&vec![]).iter().for_each(|_| {
            day_counts.entry(n).and_modify(|v| *v += 1);
        });
    }
    println!("{:?}", day_counts);

    Ok(())
}
