// main.rs
//
mod db;
mod handlers;
mod models;
mod process_result;
mod ui;
mod ws;

use {
    db::{create_user_tables, initialize_db},
    handlers::{store_assignments, store_content, store_courses, store_grades, store_pages},
    models::response::CustomError,
    models::user::{init, store_user},
    process_result::ProcessResult,
    termimad::{crossterm::style::Color::*, MadSkin, Question, *},
    ui::parser::fetch_and_print_modules,
    ws::ApiConfig,
};

enum UserCommand {
    Init,
    Section,
    Fetch,
    Default,
}

#[tokio::main]
async fn main() -> Result<(), CustomError> {
    let skin = make_skin();
    let command = prompt_command(&skin)?;

    let mut conn = initialize_db()?;
    create_user_tables(&conn)?;

    let mut api_config = if let UserCommand::Init = command {
        init(&mut conn).await?
    } else {
        ApiConfig::get_saved_api_config(&conn)?
    };

    let test_course = 29737;

    match command {
        UserCommand::Init => {
            store_user(&mut conn, &mut api_config).await?;
            let mut api_config = ApiConfig::get_saved_api_config(&conn)?;
            store_courses(&mut conn, &mut api_config).await?;
        }
        UserCommand::Section => {
            fetch_and_print_modules(&conn, test_course)?;
        }
        UserCommand::Fetch => {
            store_grades(&mut conn, &mut api_config, test_course).await?;
            store_assignments(&mut conn, &mut api_config, test_course).await?;
            store_content(&mut conn, &mut api_config, test_course).await?;
            store_pages(&mut conn, &mut api_config).await?;
        }
        UserCommand::Default => {
            fetch_and_print_modules(&conn, test_course)?;
        }
    }

    Ok(())
}

fn prompt_command(skin: &MadSkin) -> Result<UserCommand, CustomError> {
    let mut q = Question::new("Choose a command to run:");
    q.add_answer("i", "**I**nit - Initialize user information\n\tEnsure 'Secrets.toml' has your Moodle Mobile Service Key and URL.\n\tThen delete courses you are not interested in from 'Secrets.toml'.");
    q.add_answer("s", "**S**ection - Handle sections");
    q.add_answer("f", "**F**etch - Fetch resources from moodle");
    q.add_answer("d", "Default - Run the default commands");
    let a = q.ask(skin)?;

    match a.as_str() {
        "i" => Ok(UserCommand::Init),
        "s" => Ok(UserCommand::Section),
        "f" => Ok(UserCommand::Fetch),
        _ => Ok(UserCommand::Default),
    }
}

fn make_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.table.align = Alignment::Center;
    skin.set_headers_fg(AnsiValue(178));
    skin.bold.set_fg(Yellow);
    skin.italic.set_fg(Magenta);
    skin.scrollbar.thumb.set_fg(AnsiValue(178));
    skin.code_block.align = Alignment::Center;
    skin
}
