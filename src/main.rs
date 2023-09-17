// main.rs
//
#![feature(string_remove_matches)]
mod commands;
mod db;
mod downloader;
mod models;
mod parser;
mod ui;
mod utils;
mod ws;

use {
    crate::commands::{
        command::{Command, DefaultCommand},
        download::DownloadCommand,
        fetch::FetchCommand,
        init::InitCommand,
        parse::ParseCommand,
    },
    crate::db::*,
    crate::models::{configs::*, courses::*},
    crate::utils::*,
    crate::ws::*,
    eyre::Result,
    termimad::{MadSkin, Question},
};

enum UserCommand {
    Init,
    Fetch,
    Parse,
    Download,
    Default,
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_logger().expect("Failed to initialize logging");
    initialize_db()?;

    let mut config = Configs::new()?;
    let skin = make_skin();
    let command_enum = prompt_command(&skin)?;

    let client;

    let mut command: Box<dyn Command>;

    command = match command_enum {
        UserCommand::Init => Box::new(InitCommand::new(&mut config, &skin)),
        UserCommand::Fetch => {
            client = ApiClient::from_config(&config)?;
            Box::new(FetchCommand::new(client, &config))
        }
        UserCommand::Parse => Box::new(ParseCommand::new(&config)),
        UserCommand::Download => {
            client = ApiClient::from_config(&config)?;
            Box::new(DownloadCommand::new(client, &config))
        }
        UserCommand::Default => {
            client = ApiClient::from_config(&config)?;
            Box::new(DefaultCommand::new(&config, client))
        }
    };

    command.execute().await?;

    Ok(())
}

fn prompt_command(skin: &MadSkin) -> Result<UserCommand> {
    let mut q = Question::new("Choose a command to run:");
    q.add_answer(
        "i",
        "**I**nit - Initialize user information
        Ensure 'config.toml' has your Moodle Mobile Service Key and URL.",
    );
    q.add_answer(
        "f",
        "**F**etch - Fetch course material
        This will populate 'moodl-rs.db' with all course material",
    );
    q.add_answer(
        "D",
        "**D**ownload - Downloads all course materials(.pdfs, .pptxs, etc.)
        Default location is ~/ on linux/mac and typically C:\\Users\\<YourUserName> on windows
        Set the path for each course in 'config.toml' to save materials elsewhere",
    );
    q.add_answer(
        "p",
        "**P**arse - Parse the course page to a markdown file
        Default location is ~/ on linux/mac and typically C:\\Users\\<YourUserName> on windows
        Set the path for each course in 'config.toml' to save markdown elsewhere",
    );
    q.add_answer("d", "Default - Run fetch, download, parse sequentially");
    let a = q.ask(skin)?;

    match a.as_str() {
        "i" => Ok(UserCommand::Init),
        "f" => Ok(UserCommand::Fetch),
        "D" => Ok(UserCommand::Download),
        "p" => Ok(UserCommand::Parse),
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
