use command::Command;
use commands::{fish::Fish, hug::Hug, meimei::Meimei, ping::Ping, ship::Ship};
use log::{error, info};
use nom::{
    bytes::complete::{tag, take_while},
    IResult,
};
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{
        channel::Message,
        id::GuildId,
        prelude::{Activity, Ready},
    },
    Client,
};
use sqlx::{sqlite::SqlitePoolOptions, Error, Pool, Sqlite};

type CommandMaker = fn(&Message, &str) -> Result<Box<dyn Command + Send + Sync>, ()>;

mod command;
mod commands;
mod vars;

struct Handler {
    command_handlers: Vec<CommandMaker>,
    db_pool: Pool<Sqlite>,
}

struct HandlerBuilder {
    command_handlers: Vec<CommandMaker>,
    db_pool: Option<Pool<Sqlite>>,
}

impl HandlerBuilder {
    fn new() -> Self {
        Self {
            command_handlers: Vec::new(),
            db_pool: None,
        }
    }

    fn add_handler<T: 'static + Command + Send + Sync>(mut self) -> Self {
        self.command_handlers.push(|msg, cmd| {
            let cmd = T::from_msg(msg, cmd)?;
            Ok(Box::new(cmd))
        });
        self
    }

    fn add_pool(mut self, pool: Pool<Sqlite>) -> Self {
        self.db_pool = Some(pool);
        self
    }

    fn build(self) -> Handler {
        Handler {
            command_handlers: self.command_handlers,
            db_pool: self.db_pool.expect("add pool with add_pool first"),
        }
    }
}

async fn get_prefix(pool: &Pool<Sqlite>, guild: GuildId) -> Result<String, Error> {
    let id = *guild.as_u64() as i64;

    struct Prefix {
        prefix: String,
    }
    let prefix = sqlx::query_as!(
        Prefix,
        "SELECT prefix FROM ServerConfigs WHERE server_id = ? ",
        id
    )
    .fetch_optional(pool)
    .await?;

    if let Some(prefix) = prefix {
        // Ok(prefix.prefix)
        Ok(prefix.prefix)
    } else {
        let prefix = "dd".to_string();
        sqlx::query!(
            "INSERT INTO ServerConfigs (server_id, prefix) VALUES (?, ?)",
            id,
            prefix
        )
        .execute(pool)
        .await?;

        Ok(prefix)
    }
}

fn take_prefix<'a>(content: &'a str, prefix: &str) -> IResult<&'a str, ()> {
    let (content, _) = tag(prefix)(content)?;
    let (content, _) = take_while(char::is_whitespace)(content)?;

    Ok((content, ()))
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let guild_id = if let Some(guild_id) = msg.guild_id {
            guild_id
        } else {
            return;
        };
        let prefix = get_prefix(&self.db_pool, guild_id).await.unwrap();
        let content = msg.content.to_lowercase();

        let content = if let Ok((content, _)) = take_prefix(content.as_str(), &prefix) {
            content
        } else {
            return;
        };

        info!("command str: `{content}`");

        for command_handler in &self.command_handlers {
            if let Ok(command) = command_handler(&msg, content) {
                // is only sending a reference to the database which should be manageable
                if let Err(why) = command.handle(&ctx, &msg, &self.db_pool).await {
                    error!(
                        "(guild={:?}) (msg='{}') {why:?}",
                        msg.guild_id, &msg.content
                    );
                }
                return;
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_activity(Activity::playing("with you(r balls)"))
            .await;
        ctx.idle().await;
        info!("{} is connected!", ready.user.name);
    }
}

async fn setup_database() -> Result<Pool<Sqlite>, Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(std::env!("DATABASE_URL_ACTUAL"))
        .await?;

    Ok(pool)
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let token = vars::DISCORD_TOKEN;

    // setup logger
    if let Err(why) = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(std::io::stdout())
        .level_for("bot", log::LevelFilter::Trace)
        .level(log::LevelFilter::Info)
        .level_for("tracing", log::LevelFilter::Warn)
        .chain(fern::log_file("bot.log").unwrap())
        .apply()
    {
        eprintln!("failed to setup logger due to {why:?}");
    };

    let pool = match setup_database().await {
        Err(why) => {
            error!("error setting up database: {why:?}");
            panic!();
        }
        Ok(pool) => pool,
    };

    let mut client = Client::builder(token)
        .event_handler(
            HandlerBuilder::new()
                .add_handler::<Ping>()
                .add_handler::<Meimei>()
                .add_handler::<Fish>()
                .add_handler::<Hug>()
                .add_handler::<Ship>()
                .add_pool(pool)
                .build(),
        )
        .await
        .expect("error creating client");

    if let Err(why) = client.start().await {
        error!("error with client: {why:?}");
    }
}
