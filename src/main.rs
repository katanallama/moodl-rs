// main.rs
//
mod db;
mod downloader;
mod handlers;
mod models;
mod parser;
mod ui;
mod utils;
mod ws;

use {
    crate::db::*,
    crate::handlers::*,
    crate::models::{configs::*, courses::*},
    crate::utils::*,
    crate::ws::*,
};
use {
    eyre::Result,
    termimad::{MadSkin, Question},
};

enum UserCommand {
    Init,
    Parse,
    Fetch,
    Download,
    Default,
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_logger().expect("Failed to initialize logging");
    initialize_db()?;

    let mut config = Configs::new()?;
    let skin = make_skin();
    let command = prompt_command(&skin)?;

    let mut client;
    match command {
        UserCommand::Init => {
            let conn = connect_db()?;
            db::create_tables(&conn)?;

            config.prompt_config(&skin).await?;

            client = ApiClient::from_config(&config)?;
            let userid = get_userid(&mut client).await?;
            config.write_userid(userid)?;

            client = ApiClient::from_config(&config)?;
            get_courses(&skin, &mut client, &mut config).await?;
        }
        UserCommand::Fetch => {
            let mut conn = connect_db()?;
            client = ApiClient::from_config(&config)?;
            fetch_course_handler(config, &mut client, &mut conn).await?;
            fetch_page_handler(&mut client, &mut conn).await?;
        }
        UserCommand::Parse => {
            let conn = connect_db()?;
            parse_command_handler(config, &conn).await?;
        }
        UserCommand::Download => {
            let conn = connect_db()?;
            client = ApiClient::from_config(&config)?;
            download_command_handler(config, &client, &conn).await?;
        }
        UserCommand::Default => {}
    }

    Ok(())
}

fn prompt_command(skin: &MadSkin) -> Result<UserCommand> {
    let mut q = Question::new("Choose a command to run:");
    q.add_answer(
        "i",
        "**I**nit - Initialize user information
        Ensure 'config.toml' has your Moodle Mobile Service Key and URL.",
    );
    q.add_answer("f", "**F**etch - Fetch course materials");
    q.add_answer("p", "**P**arse - Parse a course");
    q.add_answer("D", "**D**ownload - Download a course");
    q.add_answer("d", "Default - Run the default commands");
    let a = q.ask(skin)?;

    match a.as_str() {
        "i" => Ok(UserCommand::Init),
        "f" => Ok(UserCommand::Fetch),
        "p" => Ok(UserCommand::Parse),
        "D" => Ok(UserCommand::Download),
        _ => Ok(UserCommand::Default),
    }
}

fn prompt_courses(courses: &Vec<Course>, skin: &MadSkin) -> Result<Vec<CourseConfig>> {
    let mut selected_courses = Vec::new();

    for course in courses.iter() {
        let question = format!(
            "Track the course *{}*?",
            course.shortname.as_ref().unwrap_or(&"Unknown".to_string())
        );

        let mut q = Question::new(&question);
        q.add_answer('y', "**Y**es, track it");
        q.add_answer('n', "**N**o, skip it");
        q.set_default('y');

        let answer = q.ask(skin)?;

        if answer == "y" {
            selected_courses.push(CourseConfig::from(course));
        }
    }

    Ok(selected_courses)
}
