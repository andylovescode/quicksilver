use crate::{
    utils::GetDB,
    Context,
    Error
};
use poise::CreateReply;
use serenity::all::{CreateEmbed, User};
use std::collections::HashMap;

/// See the items in your inventory
#[poise::command(slash_command)]
pub async fn inventory(ctx: Context<'_>, user: Option<User>) -> eyre::Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let db = ctx.db("inventory").await;

    let db_user = if let Some(x) = &user {
        db.state().get_user_or_default(&x.id)
    } else {
        db.state().get_user_or_default(&ctx.author().id)
    };

    if db_user.items.is_empty() {
        ctx.say(format!("{} inventory is empty.", if let Some(x) = &user {
            format!("<@{}>'s", x.id)
        } else {
            "Your".to_string()
        })).await?;

        return Ok(());
    }

    let mut count_map = HashMap::new();

    for item in db_user.items {
        count_map.entry(item).or_insert(0u64);

        count_map.insert(item, count_map[&item] + 1);
    }

    let mut message = CreateReply::default();
    
    for (item, count) in count_map {
        let info = item.info();

        message = message.embed(
            CreateEmbed::default()
                .title(if count > 1 {
                    format!("{} **{}** *x{}*", info.name, info.rarity.name(), count)
                } else {
                    format!("{} **{}**", info.name, info.rarity.name())
                })
                .description(info.description)
                .color(info.rarity.color())
        );
    }

    ctx.send(message).await?;

    Ok(())
}
