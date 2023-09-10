// main.rs
//
#![allow(dead_code)]

mod db;
mod downloader;
mod models;
mod parser;
mod ui;
mod utils;
mod ws;

use chrono::Local;
use {
    crate::models::configs::*,
    crate::models::courses::*,
    crate::models::pages::*,
    // crate::ui::tui::ui,
    crate::ws::*,
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

const GET_ASSIGNMENTS: &str = "mod_assign_get_assignments"; // TODO implement db
const GET_CONTENTS: &str = "core_course_get_contents";
const GET_COURSES: &str = "core_enrol_get_users_courses";
const GET_GRADES: &str = "gradereport_user_get_grade_items"; // TODO implement db
const GET_PAGES: &str = "mod_page_get_pages_by_courses";
const GET_UID: &str = "core_webservice_get_site_info";

#[tokio::main]
async fn main() -> Result<()> {
    setup_logger().expect("Failed to initialize logging");

    let config = Configs::new()?;
    let mut client = ApiClient::from_config(&config)?;
    let mut conn = db::initialize_db()?;

    let skin = make_skin();
    let command = prompt_command(&skin)?;

    match command {
        UserCommand::Init => {
            db::create_tables(&conn)?;
            init_config(&skin, &mut client, config).await?;
        }
        UserCommand::Fetch => {
            fetch_command_handler(config, &mut client, &mut conn).await?;
        }
        UserCommand::Parse => {
            parse_command_handler(config, &conn)?;
        }
        UserCommand::Download => {
            download_command_handler(config, &client, &conn).await?;
        }
        UserCommand::Default => {}
    }

    Ok(())
}

async fn init_config(
    skin: &MadSkin,
    client: &mut ApiClient,
    mut config: Configs,
) -> Result<()> {
    let response = fetch_user_id(client).await?;
    if let ApiResponse::SiteInfo(info) = response {
        config.write_userid(info.userid)?;
        config = read_config("src/config.toml")?;
        *client = ApiClient::from_config(&config)?;
    }

    let response = fetch_user_courses(client).await?;
    if let ApiResponse::Course(course_list) = response {
        let selected_courses = prompt_courses(&course_list, &skin)?;
        config.write_courses(selected_courses)?;
    }
    // Ok(config)
    Ok(())
}

async fn fetch_command_handler(
    config: Configs,
    client: &mut ApiClient,
    conn: &mut Connection,
) -> Result<()> {
    for course in config.courses {
        let response = fetch_course_contents(&client, course.id).await?;
        if let ApiResponse::Sections(mut sections) = response {
            insert_sections(conn, &mut sections, course.id)?;
        }
    }

    let mut response = fetch_course_pages(&client).await?;
    if let ApiResponse::Pages(ref mut pages) = response {
        insert_pages(conn, &mut pages.pages)?;
    }
    Ok(())
}

fn parse_command_handler(config: Configs, conn: &Connection) -> Result<()> {
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

async fn fetch_course_contents(client: &ApiClient, course_id: i64) -> Result<ApiResponse> {
    let query = QueryParameters::new(client)
        .function(GET_CONTENTS)
        .courseid(course_id);
    client.fetch(query).await
}

// TODO implement the db stuff for this
async fn fetch_course_grades(client: &ApiClient, course_id: i64) -> Result<ApiResponse> {
    let query = QueryParameters::new(client)
        .function(GET_GRADES)
        .courseid(course_id);
    client.fetch(query).await
}

async fn fetch_course_pages(client: &ApiClient) -> Result<ApiResponse> {
    let query = QueryParameters::new(client).function(GET_PAGES);
    client.fetch(query).await
}

// TODO implement the db stuff for this
async fn fetch_user_assignments(client: &ApiClient, course_id: i64) -> Result<ApiResponse> {
    let query = QueryParameters::new(client)
        .function(GET_ASSIGNMENTS)
        .courseid(course_id);
    client.fetch(query).await
}

async fn fetch_user_courses(client: &ApiClient) -> Result<ApiResponse> {
    let query = QueryParameters::new(client)
        .function(GET_COURSES)
        .use_default_userid();
    client.fetch(query).await
}

async fn fetch_user_id(client: &ApiClient) -> Result<ApiResponse> {
    let query = QueryParameters::new(client).function(GET_UID);
    client.fetch(query).await
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
