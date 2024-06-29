use crate::{data::state::DBEvent, utils::GetDB, Context, Error};

/// Increments a global counter
#[poise::command(slash_command)]
pub async fn counter(ctx: Context<'_>) -> eyre::Result<(), Error> {
    ctx.defer().await?;

    let mut db = ctx.db("counter increment").await;

    db.add(DBEvent::Counter {
        user: ctx.author().id,
    })?;

    ctx.say(format!(
        "This command has been run {} times, by {} different people!",
        db.state().counter,
        db.state().people_who_counted.len()
    ))
    .await?;

    Ok(())
}
