use std::error::Error;

use nom::{
    bytes::{complete::take_while, streaming::tag},
    IResult,
};
use rand::{
    prelude::{SliceRandom, StdRng},
    SeedableRng,
};
use serenity::{async_trait, client::Context, model::channel::Message, utils::Color};

pub struct Hug {
    target: Target,
}

use crate::{command::Command, commands::util::NoId};

use super::util::Target;

const HUG_GIFS: &[&str] = &[
    "https://data.yuibot.app/reactions/hug/829034.gif",
    "https://i.pinimg.com/originals/ae/9a/4b/ae9a4b47edb3a3bdb7eedcabdb67e31f.gif",
    "https://i.pinimg.com/originals/92/36/c3/9236c316d06fc33bedf95a03763336bd.gif",
    "https://c.tenor.com/fHhPrNT1DesAAAAd/jojo-hug.gif",
    "https://i.pinimg.com/originals/d7/17/43/d7174361c2a12be0e68122c9f7fe9c78.gif",
    "https://i.pinimg.com/originals/a8/f2/f6/a8f2f612ab90fec87a14e4266d04b812.gif",
];

#[async_trait]
impl Command for Hug {
    async fn handle(
        &self,
        ctx: &Context,
        msg: &Message,
        db_pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> Result<(), Box<dyn Error>> {
        // HUG_GIFS[chrono::Local::now().timestamp_millis() as u64 % HUG_GIFS.len()] would work too
        let mut rng = StdRng::seed_from_u64(chrono::Local::now().timestamp_millis() as u64);
        let gif = *HUG_GIFS.choose(&mut rng).expect("HUG_GIFS isnt empty");

        let id = match self.target {
            Target::Uid(uid) => uid,
            Target::MaybeId(id) => msg.guild_id.ok_or(NoId)?.member(ctx, id).await?.user.id,
        };

        struct Hugs {
            hugs: i64,
        }

        let hug_count = {
            let author_id = *msg.author.id.as_u64() as i64;
            let id = *id.as_u64() as i64;
            let hugs = sqlx::query_as!(
                Hugs,
                "SELECT hugs FROM Actions WHERE actor_id = ? AND target_id = ?",
                author_id,
                id
            )
            .fetch_optional(db_pool)
            .await?;
            let hug_count = if let Some(hugs) = hugs {
                sqlx::query!(
                    "UPDATE Actions SET hugs = hugs + 1 WHERE actor_id = ? AND target_id = ?",
                    author_id,
                    id,
                )
                .execute(db_pool)
                .await?;
                hugs.hugs + 1
            } else {
                sqlx::query!(
                    "INSERT INTO Actions (actor_id, target_id, hugs) VALUES (?, ?, ?)",
                    author_id,
                    id,
                    1i64
                )
                .execute(db_pool)
                .await?;
                1
            };
            hug_count
        };

        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.add_embed(|e| {
                    e.title("you gave a hug! :>")
                        .color(Color::from_rgb(0xbd, 0xd7, 0xb8))
                        .image(gif)
                        .description(format!(
                            "<@{}> hugs <@{}>",
                            msg.author.id.as_u64(),
                            id.as_u64()
                        ))
                        .footer(|footer| {
                            footer.text(match hug_count {
                                1 => "This is your first hug!".to_string(),
                                2 => "That's your second hug!".to_string(),
                                3 => "wow THREE hugs!".to_string(),
                                10 => "tenth hug! you're doing great".to_string(),
                                69 => "69 hugs".to_string(),
                                100 => "welcome to the 100 hugs club".to_string(),
                                500 | 1000 => "this is a lot...".to_string(),
                                i => format!("You're at {i} hugs! Amazing!"),
                            })
                        })
                })
            })
            .await?;

        Ok(())
    }

    fn from_msg(msg: &Message, content: &str) -> Result<Self, ()> {
        let tmp: IResult<_, _> = tag("hug")(content);
        let (content, _) = tmp.map_err(|_| ())?;

        if let Some(first) = msg.mentions.get(0) {
            return Ok(Hug {
                target: Target::Uid(first.id),
            });
        }

        let tmp: IResult<_, _> = take_while(char::is_whitespace)(content);
        let (content, _) = tmp.map_err(|_| ())?;

        let tmp: IResult<_, _> = take_while(|ch| ('0'..='9').contains(&ch))(content);
        let (_, num) = tmp.map_err(|_| ())?;

        let id: u64 = num.parse().map_err(|_| ())?;

        Ok(Hug {
            target: Target::MaybeId(id),
        })
    }
}
