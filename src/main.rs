// main.rs
//
mod db;
mod downloader;
mod models;
mod parser;
mod ui;
mod utils;
mod ws;

use {
    crate::db::*,
    crate::models::configs::*,
    crate::models::courses::*,
    crate::models::pages::*,
    // crate::ui::tui::ui,
    crate::ws::*,
    chrono::Local,
    downloader::save_files,
    eyre::Result,
    models::course_details::parse_course_json,
    models::course_section::insert_sections,
    parser::save_markdown_to_file,
    rusqlite::Connection,
    termimad::{crossterm::style::Color::*, MadSkin, Question, *},
    utils::modify_shortname,
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
            fetch_command_handler(config, &mut client, &mut conn).await?;
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

pub async fn get_userid(client: &mut ApiClient) -> Result<i64> {
    let response = client.fetch_user_id().await?;
    if let ApiResponse::SiteInfo(info) = response {
        return Ok(info.userid);
    }
    Err(eyre::eyre!("Unexpected API response"))
}

async fn get_courses(skin: &MadSkin, client: &mut ApiClient, config: &mut Configs) -> Result<()> {
    let response = client.fetch_user_courses().await?;

    if let ApiResponse::Course(course_list) = response {
        let selected_courses = prompt_courses(&course_list, &skin)?;
        config.write_courses(selected_courses)?;
    } else {
        return Err(eyre::eyre!("Unexpected API response: {:?}", response));
    }

    Ok(())
}

async fn fetch_command_handler(
    config: Configs,
    client: &mut ApiClient,
    conn: &mut Connection,
) -> Result<()> {
    for course in config.courses {
        let response = client.fetch_course_contents(course.id).await?;
        if let ApiResponse::Sections(mut sections) = response {
            insert_sections(conn, &mut sections, course.id)?;
        }
    }

    let mut response = client.fetch_course_pages().await?;
    if let ApiResponse::Pages(ref mut pages) = response {
        insert_pages(conn, &mut pages.pages)?;
    }
    Ok(())
}

async fn parse_command_handler(config: Configs, conn: &Connection) -> Result<()> {
    for course in config.courses {
        let json = parse_course_json(&conn, course.id)?;
        if let Some(ref shortname) = course.shortname {
            let file_path = format!("out/{}", modify_shortname(&shortname));
            save_markdown_to_file(&json, &file_path)?;
        }
    }
    Ok(())
}

async fn download_command_handler(
    config: Configs,
    client: &ApiClient,
    conn: &Connection,
) -> Result<()> {
    for course in config.courses {
        let json = parse_course_json(&conn, course.id)?;
        if let Some(ref shortname) = course.shortname {
            let file_path = format!("out/{}", modify_shortname(&shortname));
            save_files(&json, &file_path, &client, &conn).await?;
        }
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

fn make_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.table.align = Alignment::Center;
    skin.set_headers_fg(AnsiValue(178));
    skin.bold.set_fg(Yellow);
    skin.italic.set_fg(Magenta);
    skin.scrollbar.thumb.set_fg(AnsiValue(178));
    skin
}

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] [{}] {}",
                Local::now().format("%H:%M:%S"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .level_for("html5ever", log::LevelFilter::Warn)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}
