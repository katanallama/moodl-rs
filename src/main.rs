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

use {
    crate::models::configs::*,
    crate::models::courses::*,
    crate::models::pages::*,
    // crate::ui::tui::ui,
    crate::ws::*,
    anyhow::Result,
    downloader::save_files,
    models::course_details::parse_course_json,
    models::course_section::insert_sections,
    parser::save_markdown_to_file,
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
    let mut config = Configs::new();
    let mut client = ApiClient::from_config(&config.as_mut().expect("No config found"))?;
    let mut conn = db::initialize_db()?;

    let skin = make_skin();
    let command = prompt_command(&skin)?;

    match command {
        UserCommand::Init => {
            db::create_tables(&conn)?;
            let response = fetch_user_id(&client).await?;
            if let ApiResponse::SiteInfo(info) = response {
                config.expect("NO UID").write_userid(info.userid)?;
                config = Ok(read_config("src/config.toml").expect("BAD"));
                client = ApiClient::from_config(&config.as_mut().expect("Incorrect Config"))?;
                println!("Userid updated in 'config.toml' to: {}\n", info.userid);
            }
            let response = fetch_user_courses(&client).await?;
            if let ApiResponse::Course(course_list) = response {
                let selected_courses = prompt_courses(&course_list, &skin)?;
                config.expect("Cant write courses").write_courses(selected_courses)?;
            }
        }
        UserCommand::Fetch => {
            for course in config.expect("No courses").courses {
                let response = fetch_course_contents(&client, course.id).await?;
                if let ApiResponse::Sections(mut sections) = response {
                    insert_sections(&mut conn, &mut sections, course.id)?;
                }
            }

            let mut response = fetch_course_pages(&client).await?;
            if let ApiResponse::Pages(ref mut pages) = response {
                insert_pages(&mut conn, &mut pages.pages)?;
            }
        }
        UserCommand::Parse => {
            for course in config.expect("No courses").courses {
                let json = parse_course_json(&conn, course.id)?;
                if let Some(ref shortname) = course.shortname {
                    let file_path = format!("out/{}", modify_shortname(&shortname));
                    save_markdown_to_file(&json, &file_path)?;
                }
            }
        }
        UserCommand::Download => {
            for course in config.expect("No courses").courses {
                let json = parse_course_json(&conn, course.id)?;
                if let Some(ref shortname) = course.shortname {
                    let file_path = format!("out/{}", modify_shortname(&shortname));
                    save_files(&json, &file_path, &client, &conn).await?;
                }
            }
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
