use serenity::{async_trait, client::Context, model::channel::Message};
use sqlx::{Pool, Sqlite};

use crate::command::Command;

pub struct Ping;

#[async_trait]
impl Command for Ping {
    async fn handle(
        &self,
        ctx: &Context,
        msg: &Message,
        _: &Pool<Sqlite>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        msg.channel_id.say(&ctx.http, "pong :>").await?;
        Ok(())
    }

    fn from_msg(_msg: &Message, s: &str) -> Result<Self, ()> {
        if s.starts_with("ping") {
            Ok(Ping)
        } else {
            Err(())
        }
    }
}
