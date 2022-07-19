use nom::{
    bytes::streaming::{tag, take_while},
    IResult,
};
use serenity::{
    async_trait,
    model::{channel::Message, id::UserId},
};
use sqlx::Executor;
use std::error::Error;

use super::util::Target;
use crate::{
    command::Command,
    commands::util::{HahaNo, NoId},
};

pub struct Ship {
    target: Option<Target>,
}

async fn display(
    ctx: &serenity::client::Context,
    msg: &Message,
    db_pool: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<(), Box<dyn Error>> {
    struct Shipment {
        shipped_with: Option<i64>,
        shipped_since: Option<chrono::NaiveDateTime>,
    }

    let shipment: Shipment = sqlx::query_as!(
        Shipment,
        "SELECT shipped_with, shipped_since FROM Users WHERE user_id = ?",
        0
    )
    .fetch_optional(db_pool)
    .await?
    .ok_or(HahaNo)?;
    todo!()
}

async fn new_ship(
    ctx: &serenity::client::Context,
    msg: &Message,
    db_pool: &sqlx::Pool<sqlx::Sqlite>,
    target: Target,
) -> Result<(), Box<dyn Error>> {
    let id = match target {
        Target::Uid(uid) => uid,
        Target::MaybeId(id) => msg.guild_id.ok_or(NoId)?.member(ctx, id).await?.user.id,
    };

    struct Shipment {
        shipped_with: Option<i64>,
    }

    let shipment: Shipment = sqlx::query_as!(
        Shipment,
        "SELECT shipped_with FROM Users WHERE user_id = ?",
        0
    )
    .fetch_optional(db_pool)
    .await?
    // this ok_or is unsound
    .ok_or(HahaNo)?;

    if shipment.shipped_with.is_some() {
        msg.channel_id.say(ctx, "you are already shipped").await?;
        return Err(Box::new(HahaNo));
    }

    let update_or_create = |a: UserId, b: UserId| {
        sqlx::query!("");
    };

    // insert new author -> target
    // insert new target -> author

    todo!()
}

#[async_trait]
impl Command for Ship {
    async fn handle(
        &self,
        ctx: &serenity::client::Context,
        msg: &Message,
        db_pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> Result<(), Box<dyn Error>> {
        match self.target {
            Some(target) => new_ship(ctx, msg, db_pool, target).await,
            None => display(ctx, msg, db_pool).await,
        }
    }

    fn from_msg(msg: &Message, content: &str) -> Result<Self, ()>
    where
        Self: Sized,
    {
        let tmp: IResult<_, _> = tag("ship")(content);
        let (content, _) = tmp.map_err(|_| ())?;

        let tmp: IResult<_, _> = take_while(char::is_whitespace)(content);
        let content = match tmp {
            Ok((content, _)) => content,
            Err(_) => return Ok(Ship { target: None }),
        };

        if let Some(first) = msg.mentions.get(0) {
            return Ok(Ship {
                target: Some(Target::Uid(first.id)),
            });
        }

        let tmp: IResult<_, _> = take_while(|ch| ('0'..='9').contains(&ch))(content);
        let num = match tmp {
            Ok((_, num)) => num,
            Err(_) => return Ok(Ship { target: None }),
        };

        let id: u64 = match num.parse::<u64>() {
            Ok(id) => id,
            Err(_) => return Ok(Ship { target: None }),
        };

        Ok(Ship {
            target: Some(Target::MaybeId(id)),
        })
    }
}
