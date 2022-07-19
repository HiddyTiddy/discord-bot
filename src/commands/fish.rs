use nom::{bytes::streaming::tag, IResult};
use serenity::{async_trait, client::Context, model::channel::Message};

use crate::command::Command;

pub struct Fish;

#[async_trait]
impl Command for Fish {
    async fn handle(
        &self,
        ctx: &Context,
        msg: &Message,
        db_pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let id = *msg.author.id.as_u64() as i64;
        struct Fishs {
            fishes: i64,
        }

        let fishs = sqlx::query_as!(Fishs, "SELECT fishes FROM Users WHERE user_id = ?", id)
            .fetch_optional(db_pool)
            .await?;

        let fish_count = if let Some(fishs) = fishs {
            sqlx::query!("UPDATE Users SET fishes = fishes + 1 WHERE user_id = ?", id,)
                .execute(db_pool)
                .await?;
            fishs.fishes + 1
        } else {
            sqlx::query!(
                "INSERT INTO Users (user_id, fishes) VALUES (?, ?)",
                id,
                1i64
            )
            .execute(db_pool)
            .await?;
            1
        };

        msg.channel_id
            .say(
                &ctx.http,
                &format!("you got a fish! you now have {fish_count}!! :fish:"),
            )
            .await?;
        Ok(())
    }

    fn from_msg(_msg: &Message, s: &str) -> Result<Self, ()> {
        let fish: IResult<_, _> = tag("fish")(s);
        match fish {
            Ok(_) => Ok(Fish),
            Err(_) => Err(()),
        }
    }
}
