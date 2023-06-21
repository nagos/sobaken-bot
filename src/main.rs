use teloxide::{
    prelude::*,
    dispatching::{
        dialogue::{
            self, 
            InMemStorage, 
        },
        UpdateHandler,
    },
    utils::command::BotCommands, 
    types::{
        InlineKeyboardMarkup, 
        InlineKeyboardButton,
        KeyboardMarkup,
        KeyboardButton,
        InputFile,
    },
};

use tokio::time;
use rand::seq::SliceRandom;

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

const TEXT_HOW_IS_MY_DOG: &str = "Как там мой собакен?";
const TEXT_ONE_MOMENT: &str = "Секундочку";
const TEXT_DROPOFF: &str = "Сдать собакена 🦮";
const TEXT_WELCOME: &str = "Добро пожаловать";
const TEXT_TASK: &str = "Что можем помочь?";
const TEXT_10M: &str = "Через 10 минут";
const TEXT_60M: &str = "Через час";
const TEXT_WEN: &str = "Когда?";
const TEXT_DROPOFF_TIME: &str = "Ждем через";
const TEXT_DROPOFF_REMINDER: &str = "Не забудьте сдать собакена!";
const TEXT_PHOTO: &str = "Собакен на прогулке";
const TEXT_PICKUP: &str = "Ждем вас снова";

const DELAY_DROPOFF: u64 = 60;
const DELAY_WALK: u64 = 300;
const DELAY_PICKUP: u64 = 300;
const DELAY_CHECK_DELAY: u64 = 10;

const WALK_PHOTOS: &[&str] = &[
    "images/00020-1378581242.jpg",
    "images/00022-632562074.jpg",
    "images/00023-1357010347.jpg",
    "images/00029-1974425078.jpg",
];

const REST_PHOTOS: &[&str] = &[
    "images/00035-3425653837.jpg",
    "images/00036-1284626168.jpg",
    "images/00037-988796557.jpg",
    "images/00038-2520346606.jpg",
];

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    Dropoff,
    Walk,
    Pickup
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Поддерживаемые команды:")]
enum Command {
    #[command(description = "этот текст.")]
    Help,
    #[command(description = "старт.")]
    Start,
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(
            case![State::Start]
                .branch(case![Command::Help].endpoint(help))
                .branch(case![Command::Start].endpoint(start))
        );

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::Dropoff].endpoint(message))
        .branch(case![State::Walk].endpoint(message))
        .branch(case![State::Pickup].endpoint(message))
        .endpoint(help);

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(Update::filter_callback_query()
            .branch(case![State::Start].endpoint(dropoff_handler))
            .branch(case![State::Dropoff].endpoint(dropoff_time_handler))
        )
        .branch(message_handler)
}

fn random_photo<'a>(list: &'a [&str]) -> &'a str {
    list.choose(&mut rand::thread_rng()).unwrap()
}

async fn message(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    if let Some(text) = msg.text() {
        match text {
            TEXT_HOW_IS_MY_DOG => {
                bot.send_message(dialogue.chat_id(), TEXT_ONE_MOMENT).await.unwrap();
                tokio::spawn(async move {
                    time::sleep(time::Duration::from_secs(DELAY_CHECK_DELAY)).await;
                    let photo = random_photo(REST_PHOTOS);
                    bot.send_photo(dialogue.chat_id(), InputFile::file(photo)).await.unwrap();
                });

            },
            _ => help(bot, dialogue, msg).await?,
        }
    }
    Ok(())
}

async fn help(bot: Bot, _dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
    Ok(())
}

async fn start(bot: Bot, _dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let buttons = [
        InlineKeyboardButton::callback(TEXT_DROPOFF, "dropoff"),
    ];
    let buttons_keyboard = [
        KeyboardButton::new(TEXT_HOW_IS_MY_DOG)
    ];
    bot.send_message(msg.chat.id, TEXT_WELCOME)
    .reply_markup(KeyboardMarkup::default().append_row(buttons_keyboard)).await.unwrap();
    bot.send_message(msg.chat.id, TEXT_TASK)
        .reply_markup(InlineKeyboardMarkup::new([buttons]))
        .await?;
    Ok(())
}

async fn dropoff_handler(bot: Bot, dialogue: MyDialogue, q: CallbackQuery) -> HandlerResult{
    log::info!("dropoff_handler");
    if let Some(data) = q.data {
        match data.as_str() {
            "dropoff" => {
                let buttons = [
                    InlineKeyboardButton::callback(TEXT_10M, "10"),
                    InlineKeyboardButton::callback(TEXT_60M, "60"),
                ];
                bot.answer_callback_query(q.id).await?;
                bot.send_message(dialogue.chat_id(), TEXT_WEN)
                    .reply_markup(InlineKeyboardMarkup::new([buttons]))
                    .await?;
                dialogue.update(State::Dropoff).await?;
            }
            _ => {}
        };
    }
    Ok(())
}

async fn dropoff_time_handler(bot: Bot, dialogue: MyDialogue, q: CallbackQuery) -> HandlerResult{
    if let Some(data) = q.data {
        let time = match data.as_str() {
            "10" => 10,
            "60" => 60,
            _ => {0}
        };
        bot.answer_callback_query(q.id).await?;
        bot.send_message(dialogue.chat_id(), format!("{} {}", TEXT_DROPOFF_TIME, time)).await?;

        tokio::spawn(async move {
            
            time::sleep(time::Duration::from_secs(DELAY_DROPOFF)).await;
            bot.send_message(dialogue.chat_id(), TEXT_DROPOFF_REMINDER).await.unwrap();
            dialogue.update(State::Walk).await.unwrap();

            tokio::spawn(async move {
                time::sleep(time::Duration::from_secs(DELAY_WALK)).await;
                bot.send_message(dialogue.chat_id(), TEXT_PHOTO).await.unwrap();
                let photo = random_photo(WALK_PHOTOS);
                bot.send_photo(dialogue.chat_id(), InputFile::file(photo)).await.unwrap();
                dialogue.update(State::Pickup).await.unwrap();

                tokio::spawn(async move {
                    time::sleep(time::Duration::from_secs(DELAY_PICKUP)).await;
                    bot.send_message(dialogue.chat_id(), TEXT_PICKUP).await.unwrap();
                    dialogue.update(State::Start).await.unwrap();
                });
            });
        });
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting sobaken bot...");

    let bot = Bot::from_env();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
