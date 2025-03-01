mod chat;

use dotenv::dotenv;
use serenity::all::{ChannelType, GuildChannel, PartialGuildChannel};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;
use std::env;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
        let channel = msg.channel(ctx.http.clone()).await.unwrap();
        if channel.clone().guild().unwrap().kind == ChannelType::PublicThread {
            if channel.clone().guild().unwrap().owner_id.unwrap() != ctx.cache.current_user().id {
                return;
            }
            let mut conv = chat::Conversation::get(ctx.clone(), channel).await;
            conv.send_msg(ctx.clone(), msg).await;
        } else if msg.mentions.contains(&**ctx.cache.current_user())
            && channel.clone().guild().unwrap().kind == ChannelType::Text
        {
            chat::Conversation::create(channel, msg, ctx.clone()).await;
        }
    }
    async fn thread_delete(
        &self,
        _: Context,
        guild: PartialGuildChannel,
        _: Option<GuildChannel>,
    ) {
        chat::Conversation::delete(guild.id).await;
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
