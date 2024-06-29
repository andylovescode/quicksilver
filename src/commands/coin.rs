use crate::{
    data::state::SideChannel,
    data::{rng::Chance, state::DBEvent},
    utils::GetDB,
    Context, Error,
};

/// Flip a coin!
#[poise::command(slash_command)]
pub async fn coinflip(ctx: Context<'_>) -> eyre::Result<(), Error> {
    ctx.defer().await?;

    let mut db = ctx.db("coin flip").await;

    let chance = Chance::new();

    let result = db.add(DBEvent::CoinFlip { chance })?;

    match result {
        SideChannel::CoinFlip { success } => {
            if success {
                ctx.say(format!(
                    "Heads! **{}** successful coin flips in a row! (that's a 1/{} chance)",
                    db.state().flips_in_a_row,
                    2u32.pow(db.state().flips_in_a_row)
                ))
                .await?;
            } else {
                ctx.say("Unfortunately, you landed on tails.").await?;
            }
        }
        event => {
            panic!("CoinFlip event returned {:?} not CoinFlip", event)
        }
    }

    Ok(())
}
