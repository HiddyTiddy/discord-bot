use std::error::Error;

use nom::{
    bytes::complete::{tag, tag_no_case, take_while},
    IResult,
};
use serenity::{async_trait, client::Context, model::channel::Message};
use sqlx::{Pool, Sqlite};
use tinyvec::ArrayVec;

#[async_trait]
pub trait Command {
    async fn handle(
        &self,
        ctx: &Context,
        msg: &Message,
        db_pool: &Pool<Sqlite>,
    ) -> Result<(), Box<dyn Error>>;

    fn from_msg(msg: &Message, content: &str) -> Result<Self, ()>
    where
        Self: Sized;
}

pub struct BaseCommand<H>
where
    H: Command,
{
    command_name: &'static str,
    alias: ArrayVec<[&'static str; 4]>,
    help_message: Option<&'static str>,
    handler: H,
}

impl<H> BaseCommand<H>
where
    H: Command,
{
    pub fn matches<'a>(&self, input: &'a str) -> IResult<&'a str, ()> {
        let mut result: IResult<&str, _> = tag(self.command_name)(input);
        for i in self.alias.iter() {
            result = result.or_else(|_| {
                let tmp: IResult<&str, _> = tag(*i)(input);
                tmp
            });
        }

        let (input, _) = result?;

        let (input, _) = take_while(char::is_whitespace)(input)?;
        Ok((input, ()))
    }
}
