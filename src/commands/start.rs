use serenity::all::ActivityData;

use crate::types::{Context, Error, OrpheusStatus};

#[tracing::instrument]
#[poise::command(slash_command)]
pub async fn start(ctx: Context<'_>) -> Result<(), Error> {
    if ctx.author().id != 378323967158517763 && ctx.author().id != 606692751752429585 {
        _ = ctx
            .reply("Sorry, only <@378323967158517763> or <@606692751752429585> can start me.")
            .await;
        return Ok(());
    }
    let mut status = ctx.data().status.lock().await;
    *status = OrpheusStatus::Waiting;
    ctx.serenity_context()
        .set_activity(Some(ActivityData::custom(status.as_str())));

    _ = ctx.reply("Ok, I'm started and waiting.").await;

    Ok(())
}
