// main.rs
//
mod db;
mod handlers;
mod models;
mod process_result;
mod ui;
mod ws;

use {
    // clap::Parser,
    db::{create_course_content_tables, create_user_table, initialize_db},
    handlers::{store_assignments, store_content, store_courses, store_grades, store_user, store_pages},
    models::response::CustomError,
    process_result::ProcessResult,
    reqwest,
    std::io::{self, Write},
    termimad::{crossterm::style::Color::*, MadSkin, Question, *},
    ui::parser::fetch_and_print_modules,
    ws::ApiConfig,
};

// New enum for termimad TUI choices
enum UserCommand {
    Init,
    Section,
    Default,
}

#[tokio::main]
async fn main() -> Result<(), CustomError> {
    let skin = make_skin();
    let command = prompt_command(&skin)?;

    let mut conn = initialize_db()?;

    let mut api_config = if let UserCommand::Init = command {
        init(&mut conn)?
    } else {
        ApiConfig::get_saved_api_config(&conn)?
    };

    let test_course = 29737;

    match command {
        UserCommand::Init => {
            store_user(&mut conn, &mut api_config).await?;
            let mut api_config = ApiConfig::get_saved_api_config(&conn)?;
            store_courses(&mut conn, &mut api_config).await?;
            create_course_content_tables(&conn)?;
        }
        UserCommand::Section => {
            fetch_and_print_modules(&conn, test_course)?;
        }
        UserCommand::Default => {
            store_grades(&mut conn, &mut api_config, test_course).await?;
            store_assignments(&mut conn, &mut api_config, test_course).await?;
            store_content(&mut conn, &mut api_config, test_course).await?;
            store_pages(&mut conn, &mut api_config).await?;
        }
    }

    Ok(())
}

fn prompt_command(skin: &MadSkin) -> Result<UserCommand, CustomError> {
    let mut q = Question::new("Choose a command to run:");
    q.add_answer("i", "**I**nit - Initialize the application");
    q.add_answer("s", "**S**ection - Handle sections");
    q.add_answer("d", "Default - Run the default commands");
    let a = q.ask(skin)?;

    match a.as_str() {
        "i" => Ok(UserCommand::Init),
        "s" => Ok(UserCommand::Section),
        _ => Ok(UserCommand::Default),
    }
}

fn init(conn: &mut rusqlite::Connection) -> Result<ApiConfig, CustomError> {
    create_user_table(conn)?;
    print!("Moodle Mobile additional features service key : ");
    io::stdout().flush()?;
    let mut wstoken = String::new();
    io::stdin().read_line(&mut wstoken)?;
    let wstoken = wstoken.trim().to_string();

    if wstoken.is_empty() {
        return Err(CustomError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Moodle key is required!",
        )));
    }

    print!("Moodle url (RTN for default) : ");
    io::stdout().flush()?;
    let mut url = String::new();
    io::stdin().read_line(&mut url)?;
    let url = if url.trim().is_empty() {
        "https://urcourses.uregina.ca/webservice/rest/server.php".to_string()
    } else {
        url.trim().to_string()
    };

    let api_config = ApiConfig {
        wstoken,
        courseid: None,
        userid: None,
        client: reqwest::Client::new(),
        url,
    };

    Ok(api_config)
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
