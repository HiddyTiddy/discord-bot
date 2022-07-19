// imports
use crate::command::Command;
use serenity::{async_trait, client::Context, model::channel::Message};
use sqlx::{Pool, Sqlite};

pub struct Meimei;

#[async_trait]
impl Command for Meimei {
    async fn handle(
        &self,
        ctx: &Context,
        msg: &Message,
        _: &Pool<Sqlite>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        const GIF: &str = "https://cdn.discordapp.com/attachments/719213703920222263/935642994906902608/801F352C-C4DA-4696-BF9E-70CDD3CE12FE.gif";
        msg.channel_id.say(&ctx.http, GIF).await?;
        Ok(())
    }

    fn from_msg(_msg: &Message, s: &str) -> Result<Self, ()> {
        if s.starts_with("meimei") {
            Ok(Meimei)
        } else {
            Err(())
        }
    }
}
