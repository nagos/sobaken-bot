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

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    Dropoff,
    Walk,
    Pickup
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "start.")]
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
        .branch(case![State::Walk].endpoint(message))
        .endpoint(help);

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(Update::filter_callback_query()
            .branch(case![State::Start].endpoint(dropoff_handler))
            .branch(case![State::Dropoff].endpoint(dropoff_time_handler))
        )
        .branch(message_handler)
}

async fn message(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    if let Some(text) = msg.text() {
        match text {
            "How is my dog?" => {
                bot.send_message(dialogue.chat_id(), "One moment").await.unwrap();
                tokio::spawn(async move {
                    time::sleep(time::Duration::from_secs(10)).await;
                    bot.send_photo(dialogue.chat_id(), InputFile::file("00020-1378581242.jpg")).await.unwrap();
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
        InlineKeyboardButton::callback("Dropoff", "dropoff"),
    ];
    let buttons_keyboard = [
        KeyboardButton::new("How is my dog?")
    ];
    bot.send_message(msg.chat.id, "Welcome")
    .reply_markup(KeyboardMarkup::default().append_row(buttons_keyboard)).await.unwrap();
    bot.send_message(msg.chat.id, "Task")
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
                    InlineKeyboardButton::callback("In 10 minutes", "10"),
                    InlineKeyboardButton::callback("In 1 hour", "60"),
                ];
                bot.answer_callback_query(q.id).await?;
                bot.send_message(dialogue.chat_id(), "When")
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
    log::info!("dropoff_time_handler");
    if let Some(data) = q.data {
        let time = match data.as_str() {
            "10" => 10,
            "60" => 60,
            _ => {0}
        };
        bot.answer_callback_query(q.id).await?;
        bot.send_message(dialogue.chat_id(), format!("Drop off in {time} minutes")).await?;

        tokio::spawn(async move {
            
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            bot.send_message(dialogue.chat_id(), "Dropoff reminder").await.unwrap();
            dialogue.update(State::Walk).await.unwrap();

            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                bot.send_message(dialogue.chat_id(), "Photo").await.unwrap();
                bot.send_photo(dialogue.chat_id(), InputFile::file("00020-1378581242.jpg")).await.unwrap();
                dialogue.update(State::Pickup).await.unwrap();

                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    bot.send_message(dialogue.chat_id(), "Pickup").await.unwrap();
                    dialogue.update(State::Pickup).await.unwrap();
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
