use serenity::all::{ActivityData, UserId};

use crate::discord::thread;
use crate::telephone::Telephone;
use crate::types::{Context, Error, OrpheusStatus};

#[tracing::instrument]
#[poise::command(slash_command, prefix_command)]
pub async fn set_lead(
    ctx: Context<'_>,
    #[description = "User"]
    #[rest]
    user: String,
) -> Result<(), Error> {
    let mut status = ctx.data().status.lock().await;
    *status = OrpheusStatus::Processing;
    ctx.serenity_context()
        .set_activity(Some(ActivityData::custom("Processing...")));

    let (telephone_option, thread_message) = thread::get::<Telephone>(ctx).await;
    let mut telephone = telephone_option.unwrap();
    println!("{:?}", telephone);
    if telephone.host != ctx.author().id {
        let _ = ctx
            .reply(format!(
                "Sorry, only the host (<@{}>) can set the story lead",
                telephone.host
            ))
            .await;
        return Ok(());
    }

    let re = regex::Regex::new(r"<@(\d+)>").unwrap();
    let user_id = re
        .captures_iter(&user)
        .filter_map(|c| c.get(1))
        .map(|m| m.as_str())
        .filter_map(|s| s.parse::<u64>().ok())
        .next();
    match user_id {
        Some(id) => telephone.lead = Some(UserId::new(id)),
        None => {
            let _ = ctx
                .reply("You must supply a valid user to take the lead!")
                .await;
            return Ok(());
        }
    }

    let _ = thread::update(
        ctx.http(),
        thread_message.channel_id,
        thread_message.id,
        telephone,
    )
    .await;

    *status = OrpheusStatus::Waiting;
    ctx.serenity_context()
        .set_activity(Some(ActivityData::custom(status.as_str())));

    Ok(())
}
