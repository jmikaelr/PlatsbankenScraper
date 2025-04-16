#![allow(dead_code)]
use crate::constants;
use crate::database::{store_user, DbJobAd, User};
use crate::find_jobs::{OccupationType, Region};
use crate::logging::info;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::IntoEnumIterator;

use std::collections::HashSet;

use dotenv::dotenv;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    utils::command::BotCommands,
};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct UserSelections {
    pub selected_occupations: Option<HashSet<OccupationType>>,
    pub selected_regions: Option<HashSet<Region>>,
}

impl UserSelections {
    pub fn new() -> Self {
        Self {
            selected_occupations: Some(HashSet::new()),
            selected_regions: Some(HashSet::new()),
        }
    }

    fn toggle(&mut self, occupation: OccupationType) {
        if let Some(set) = self.selected_occupations.as_mut() {
            if !set.insert(occupation.clone()) {
                set.remove(&occupation);
            }
        } else {
            self.selected_occupations = Some(std::collections::HashSet::from([occupation]));
        }
    }

    fn is_selected(&self, occupation: &OccupationType) -> bool {
        if let Some(set) = &self.selected_occupations {
            set.contains(occupation)
        } else {
            false
        }
    }
}

pub async fn setup_bot() -> Bot {
    dotenv().ok();
    Bot::from_env()
}

pub async fn run_bot() {
    let bot = setup_bot().await;
    Command::repl(bot, answer).await;
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "Dessa kommandot är tillgängliga!"
)]
enum Command {
    Start,
    Hjälp,
    Bevaka,
    Bevakningar,
    Prenumeration,
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Start => {
            if let Some(user) = msg.from {
                if user.is_bot {
                    return Ok(());
                }
                let user_id = user.id;
                store_user(User::new(user_id.to_string())).await;
            }
            let welcome_message = "👋 Välkommen till Platsbanken JobbBot! 🎉\n\n\
                                Jag är här för att hjälpa dig att hitta de senaste jobbmöjligheterna från Platsbanken direkt i din Telegram-app. Här är vad jag kan göra:\n\n\
                                🔍 Sök jobb: Ange dina önskemål för att hitta relevanta jobbannonser.\n\n\
                                📌 Spara sökningar: Få uppdateringar när nya jobb dyker upp som matchar dina kriterier.\n\n\
                                📅 Dagliga uppdateringar: Håll dig uppdaterad med de senaste jobben varje dag.\n\n\
                                För att komma igång, skriv bara /hjälp för att se alla kommandon jag erbjuder!\n\n\
                                Lycka till med ditt jobbsökande! 👩‍💻👨‍💻";
            bot.send_message(msg.chat.id, welcome_message).await?
        }
        Command::Hjälp => {
            let descriptions = r#"
            🤖 *Botens Kommandocenter* 🚀

            /start - Starta boten och få en snabb introduktion 🟢
            /hjälp - Visa denna hjälptext 📖
            /bevaka - Börja bevaka något specifikt (t.ex. ett jobb) 👀
            /bevakningar - Visa alla dina aktiva bevakningar 📝
            /prenumeration - Prenumerera på uppdateringar och få notiser 🔔

            *Tips*: Skriv ett kommando och följ instruktionerna för att använda boten. Behöver du mer hjälp? Bara fråga mig! ✨
            "#;

            let escaped_descriptions = escape_markdown(descriptions);

            bot.send_message(msg.chat.id, escaped_descriptions)
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?
        }

        Command::Bevaka => {
            let message = handle_bevaka(bot.clone(), msg.clone()).await?;
            return Ok(());
        }

        Command::Bevakningar => {
            todo!();
        }
        Command::Prenumeration => {
            todo!();
        }
    };

    Ok(())
}

async fn handle_bevaka(bot: Bot, msg: Message) -> ResponseResult<()> {
    let occupations = OccupationType::iter().collect::<Vec<_>>();
    let mut user_selections = UserSelections::new();

    update_keyboard(&bot, &msg, &occupations, &mut user_selections).await
}

async fn update_keyboard(
    bot: &Bot,
    msg: &Message,
    occupations: &[OccupationType],
    user_selections: &mut UserSelections,
) -> ResponseResult<()> {
    let buttons = occupations
        .iter()
        .map(|occupation| {
            let label = if user_selections.is_selected(occupation) {
                format!("{} ✅", occupation.as_readable_string())
            } else {
                format!("{} ⬜️", occupation.as_readable_string())
            };
            InlineKeyboardButton::callback(label, occupation.as_readable_string())
        })
        .collect::<Vec<_>>();

    let mut rows = buttons
        .chunks(2)
        .map(|chunk| chunk.to_vec())
        .collect::<Vec<_>>();

    rows.push(vec![InlineKeyboardButton::callback("Klar ✅", "done")]);
    println!("{user_selections:?}");

    let keyboard = InlineKeyboardMarkup::new(rows);

    bot.send_message(msg.chat.id, "Välj yrkeskategori(er) du vill bevaka:")
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

fn escape_markdown(text: &str) -> String {
    text.replace('_', "\\_")
        .replace('*', "\\*")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace('~', "\\~")
        .replace('`', "\\`")
        .replace('>', "\\>")
        .replace('#', "\\#")
        .replace('+', "\\+")
        .replace('-', "\\-")
        .replace('=', "\\=")
        .replace('|', "\\|")
        .replace('.', "\\.")
        .replace('!', "\\!")
}
