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
        KeyboardRemove,
        InputFile,
    },
};

use tokio::time;
use rand::seq::SliceRandom;

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerError = Box<dyn std::error::Error + Send + Sync>;
type HandlerResult = Result<(), HandlerError>;

const TEXT_HOW_IS_MY_DOG: &str = "Как там мой собакен?";
const TEXT_ONE_MOMENT: &str = "Секундочку";
const TEXT_DROPOFF: &str = "Сдать собакена 🦮";
const TEXT_WELCOME: &str = "Добро пожаловать";
const TEXT_TASK: &str = "Что можем помочь?";
const TEXT_10M: &str = "Через 10 минут";
const TEXT_60M: &str = "Через час";
const TEXT_WHEN: &str = "Когда?";
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
    "images/00065-1572139399.jpg",
    "images/00066-2446649328.jpg",
    "images/00070-3953615447.jpg",
    "images/00073-4245297097.jpg",
    "images/00075-1564281374.jpg",
    "images/00076-1009691694.jpg",
];

const REST_PHOTOS: &[&str] = &[
    "images/00035-3425653837.jpg",
    "images/00036-1284626168.jpg",
    "images/00037-988796557.jpg",
    "images/00038-2520346606.jpg",
    "images/00078-1317528941.jpg",
    "images/00080-472880983.jpg",
    "images/00081-2778530334.jpg",
    "images/00082-4225865963.jpg",
    "images/00083-981580161.jpg",
    "images/00084-1287769773.jpg",
];

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    Dropoff,
    DropoffTime,
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

fn schema() -> UpdateHandler<HandlerError> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(case![Command::Help].endpoint(help_handler))
        .branch(case![Command::Start].endpoint(start_handler)
    );

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::Dropoff].endpoint(message))
        .branch(case![State::Walk].endpoint(message))
        .branch(case![State::Pickup].endpoint(message))
        .endpoint(help);

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(Update::filter_callback_query()
            .branch(case![State::Dropoff].endpoint(dropoff_handler))
            .branch(case![State::DropoffTime].endpoint(dropoff_time_handler))
        )
        .branch(message_handler)
}

fn random_photo<'a>(list: &'a [&str]) -> &'a str {
    list.choose(&mut rand::thread_rng()).expect("non empty list")
}

async fn message(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    if let Some(text) = msg.text() {
        if text == TEXT_HOW_IS_MY_DOG {
                log::info!("Rest photo");
                bot.send_message(dialogue.chat_id(), TEXT_ONE_MOMENT).await?;
                time::sleep(time::Duration::from_secs(DELAY_CHECK_DELAY)).await;
                let photo = random_photo(REST_PHOTOS);
                bot.send_photo(dialogue.chat_id(), InputFile::file(photo)).await?;
        } else {
            help(&bot, &dialogue, &msg).await?;
        }
    }
    Ok(())
}

async fn help_handler(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    help(&bot, &dialogue, &msg).await
}

async fn help(bot: &Bot, _dialogue: &MyDialogue, msg: &Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
    Ok(())
}

async fn start_handler(bot: Bot, dialogue: MyDialogue) -> HandlerResult {
    start(&bot, &dialogue).await
}

async fn start(bot: &Bot, dialogue: &MyDialogue) -> HandlerResult {
    let buttons = [
        InlineKeyboardButton::callback(TEXT_DROPOFF, "dropoff"),
    ];

    log::info!("Start");
    bot.send_message(dialogue.chat_id(), TEXT_WELCOME)
        .reply_markup(KeyboardRemove::default()).await?;
    bot.send_message(dialogue.chat_id(), TEXT_TASK)
        .reply_markup(InlineKeyboardMarkup::new([buttons]))
        .await?;
    dialogue.update(State::Dropoff).await?;
    Ok(())
}

async fn dropoff_handler(bot: Bot, dialogue: MyDialogue, q: CallbackQuery) -> HandlerResult {
    log::info!("Dropoff handler");
    if let Some(data) = q.data {
        if data.as_str() == "dropoff" {
            log::info!("Dropoff time");
            let buttons = [
                InlineKeyboardButton::callback(TEXT_10M, "10"),
                InlineKeyboardButton::callback(TEXT_60M, "60"),
            ];
            bot.answer_callback_query(q.id).await?;
            bot.send_message(dialogue.chat_id(), TEXT_WHEN)
                .reply_markup(InlineKeyboardMarkup::new([buttons]))
                .await?;
            dialogue.update(State::DropoffTime).await?;
        }
    }
    Ok(())
}

async fn dropoff_time_handler(bot: Bot, dialogue: MyDialogue, q: CallbackQuery) -> HandlerResult {
    if let Some(data) = q.data {
        log::info!("Dropoff");
        let time = match data.as_str() {
            "10" => 10,
            "60" => 60,
            _ => {0}
        };
        bot.answer_callback_query(q.id).await?;
        bot.send_message(dialogue.chat_id(), format!("{} {}", TEXT_DROPOFF_TIME, time)).await?;

        tokio::spawn(async move {
            time::sleep(time::Duration::from_secs(DELAY_DROPOFF)).await;
            dropoff_reminder_msg(&bot, &dialogue).await.unwrap();

            time::sleep(time::Duration::from_secs(DELAY_WALK)).await;
            walk_photo(&bot, &dialogue).await.unwrap();

            time::sleep(time::Duration::from_secs(DELAY_PICKUP)).await;
            end_msg(&bot, &dialogue).await.unwrap();
        });
    }
    Ok(())
}

async fn dropoff_reminder_msg(bot: &Bot, dialogue: &MyDialogue) -> HandlerResult {
    let buttons_keyboard = [
        KeyboardButton::new(TEXT_HOW_IS_MY_DOG)
    ];
    log::info!("Dropoff reminder");
    bot.send_message(dialogue.chat_id(), TEXT_DROPOFF_REMINDER)
        .reply_markup(
            KeyboardMarkup::default()
            .resize_keyboard(true)
            .append_row(buttons_keyboard))
        .await?;
    dialogue.update(State::Walk).await?;
    Ok(())
}

async fn walk_photo(bot: &Bot, dialogue: &MyDialogue) -> HandlerResult {
    log::info!("Walk photo");
    bot.send_message(dialogue.chat_id(), TEXT_PHOTO).await?;
    let photo = random_photo(WALK_PHOTOS);
    bot.send_photo(dialogue.chat_id(), InputFile::file(photo)).await?;
    dialogue.update(State::Pickup).await?;
    Ok(())
}

async fn end_msg(bot: &Bot, dialogue: &MyDialogue) -> HandlerResult {
    log::info!("End");
    bot.send_message(dialogue.chat_id(), TEXT_PICKUP).await?;
    dialogue.update(State::Start).await?;
    start(bot, dialogue).await?;
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
