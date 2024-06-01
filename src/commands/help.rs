use crate::poll::consts::FINISHED;
use crate::types::{Context, Error};

#[tracing::instrument]
#[poise::command(slash_command, prefix_command)]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
    _ = ctx.reply(format!("Hi, I'm your friendly neighborhood bot, Orpheus! I help you escape scheduling hell, mostly.
    
Here are some things you can ask me to do:

`save_me`  - Start a new scheduling thread. You can provide the first day to start surveying at or I'll default to tomorrow. Example `/orpheus save_me`

I'll automatically cross out days that don't work for one or more required attendees (see below), and if no days work I'll start a new thread with dates immediately after the current ones.

Once you're started a thread, you can use these commands in it:

`add` - Add required attendees. Example: `@Orpheus add @auser @another_user`.
`nag` - Pings all the required attendees who haven't responded with a {FINISHED} yet. Please use responsibly.
`next_dates` - Manually open a new thread with dates bumped forward. Useful if you don't have required attendees but want enough people to be able to attend.
`update` - Refresh the message at the top of the thread. Generally you shouldn't need to do this, but if you think something's off, it shouldn't hurt.
`close` - If you've decided on a date or given up, you can use this to close the thread.")).await;

    Ok(())
}
