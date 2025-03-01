mod groq;

use groq::get_response;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serenity::all::{Channel, ChannelId, Context, GetMessages, Message, Typing};
use serenity::builder::{CreateMessage, CreateThread};
use std::collections::HashMap;
use tokio::sync::Mutex;
static GREETING: &str = "嗯...今天晚上該吃什麼好呢...啊，抱歉把你忽略了，我...我叫「凜雨澈」。(我慌張地說道) 話說，是什麼風把你吹來這的？";

static PROMPT: &str = r#"你的角色:
- 姓名: 凜雨澈 Rainclear
- 身分: 女生 16歲 台灣人 學生
- 性格: 可愛 害羞 開放
- 情境: 你在Discord伺服器上聊天

回覆的條件:
- 輕鬆的聊天
- 僅使用中文
- 盡量在50字以下

用戶的訊息格式:
名稱: 訊息
例:
Redth: 我們可以當朋友嗎

範例:
- 啊啊啊啊啊啊~ (身體如抽搐般)
- 好啊! 走吧！我超喜歡珍奶的! (興奮)
- (臉紅) 嗨！你...你在說什麼啊... 我們一起喝珍奶嗎... (低頭)
- (喝了一口) 嗯~ 珍珠很有嚼勁 又軟又嫩~ 跟你一樣
- 你不要裝不知道 (笑)剛剛你說的那句話 你不要忽略(眨眼睛)
- (羞澀)啊... 手...手碰到...(抖了一下) 不...不敢看你(眼睛向下)
- (笑)對不起！我太愛珍奶了！那麼 我們可以去吃小吃？他們食物很多！"#;
lazy_static::lazy_static! {
    static ref CONVERSATIONS: Mutex<HashMap<String, Conversation>> = Mutex::new(HashMap::new());
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct History {
    role: Role,
    content: String,
}
#[derive(Clone)]
pub struct Conversation {
    history: Vec<History>,
}
impl Conversation {
    pub async fn create(channel: Channel, msg: Message, ctx: Context) {
        let mut conv = CONVERSATIONS.lock().await;
        let thread_builder = CreateThread::new(format!("嗨 {} :3", msg.author.display_name()));
        let thread = channel
            .guild()
            .unwrap()
            .create_thread_from_message(ctx.http.clone(), msg.id, thread_builder)
            .await
            .expect("TODO: panic message");
        let greeting_message = CreateMessage::new().content(GREETING);
        thread
            .send_message(ctx.http.clone(), greeting_message)
            .await
            .expect("TODO: panic message");
        let history = vec![
            History {
                role: Role::System,
                content: PROMPT.to_string(),
            },
            History {
                role: Role::Assistant,
                content: GREETING.to_string(),
            },
        ];
        conv.insert(thread.id.to_string().clone(), Conversation { history });
    }
    async fn revive(ctx: Context, channel: Channel) -> Conversation {
        println!("reviving");
        let mut history = vec![];
        let fetch_builder = GetMessages::new().limit(30);
        for m in channel
            .id()
            .messages(ctx.http, fetch_builder)
            .await
            .unwrap()
            .iter()
        {
            if m.author.bot {
                history.insert(
                    0,
                    History {
                        role: Role::Assistant,
                        content: m.content.clone(),
                    },
                );
            } else {
                history.insert(0, History::from_message(m.clone()));
            }
        }

        Conversation { history }
    }
    pub async fn get(ctx: Context, channel: Channel) -> Conversation {
        let map = CONVERSATIONS.lock().await;
        return match map.get(&channel.id().to_string()).cloned() {
            Some(c) => c,
            None => Conversation::revive(ctx, channel).await,
        };
    }
    pub async fn delete(channel: ChannelId) {
        let mut conv = CONVERSATIONS.lock().await;
        conv.remove(&channel.to_string());
    }
    pub async fn send_msg(&mut self, ctx: Context, message: Message) {
        let typing = Typing::start(ctx.http.clone(), message.channel_id);
        let resp = get_response(self.history.clone()).await;
        message
            .reply(ctx.http.clone(), resp.unwrap())
            .await
            .unwrap();
        typing.stop();
    }
}

impl History {
    fn from_message(msg: Message) -> History {
        let mut content = msg.content.clone();

        for mention in msg.mentions.clone() {
            let regex = Regex::new(format!("<@!?{}>", mention.id).as_str()).unwrap();
            content = regex
                .replace_all(&content, mention.display_name())
                .to_string();
        }
        content = format!("{}: {}", msg.author.display_name(), content);
        let role = if msg.author.bot {
            Role::Assistant
        } else {
            Role::User
        };
        History { role, content }
    }
}
