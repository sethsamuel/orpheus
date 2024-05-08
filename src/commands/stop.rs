use serenity::all::{ActivityData, GetMessages};

use crate::poll::Poll;
use crate::types::{Context, Error, OrpheusStatus};

#[tracing::instrument]
#[poise::command(prefix_command)]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    if ctx.author().id != 378323967158517763 && ctx.author().id != 606692751752429585 {
        let _ = ctx
            .reply("Sorry, only <@378323967158517763> or <@606692751752429585> can stop me.")
            .await;
        return Ok(());
    }
    let mut status = ctx.data().status.lock().await;
    *status = OrpheusStatus::Stopped;
    ctx.serenity_context()
        .set_activity(Some(ActivityData::custom("Stopped")));

    let _ = ctx
        .reply("Ok, stopped until you ask me to `start` again.")
        .await;

    Ok(())
}
