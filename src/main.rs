// main.rs
mod db;
mod models;
mod process_result;
mod ws;

use crate::models::user::process_user;
use clap::Parser;
use db::{
    create_course_content_tables, create_user_table, initialize_db, insert_assignments,
    insert_content, insert_grades, insert_user,
};
use models::course_content::{process_assignments, process_content, process_grades};
use models::response::CustomError;
use process_result::ProcessResult;
use reqwest;
use std::io::{self, Write};
use ws::ApiConfig;

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    cmd: Option<Command>,
}

#[derive(Parser)]
enum Command {
    Init,
}

#[tokio::main]
async fn main() -> Result<(), CustomError> {
    let args = Cli::parse();
    let mut conn = initialize_db()?;

    let mut api_config = if let Some(Command::Init) = args.cmd {
        init(&mut conn)?
    } else {
        ApiConfig::get_saved_api_config(&conn)?
    };

    if let Some(Command::Init) = args.cmd {
        store_user(&mut conn, &mut api_config).await?;
    } else {
        create_course_content_tables(&conn)?;
        store_grades(&mut conn, &mut api_config, 26490).await?;
        store_assignments(&mut conn, &mut api_config, 26490).await?;
        store_content(&mut conn, &mut api_config, 26490).await?;
    };

    Ok(())
}

async fn store_grades(
    conn: &mut rusqlite::Connection,
    api_config: &mut ApiConfig,
    course_id: i32,
) -> Result<(), CustomError> {
    api_config.courseid = Some(course_id);
    if let ProcessResult::Grades(grades) = api_config
        .call_json(conn, "gradereport_user_get_grade_items", process_grades)
        .await?
    {
        insert_grades(conn, &grades)?;
    }

    Ok(())
}

async fn store_content(
    conn: &mut rusqlite::Connection,
    api_config: &mut ApiConfig,
    course_id: i32,
) -> Result<(), CustomError> {
    api_config.userid = None;
    api_config.courseid = Some(course_id);
    if let ProcessResult::Content(cont) = api_config
        .call_json(conn, "core_course_get_contents", process_content)
        .await?
    {
        insert_content(conn, api_config.courseid, &cont)?;
    }

    Ok(())
}

async fn store_assignments(
    conn: &mut rusqlite::Connection,
    api_config: &mut ApiConfig,
    _course_id: i32,
) -> Result<(), CustomError> {
    api_config.courseid = None;
    api_config.userid = None;
    if let ProcessResult::Assigns(assigns) = api_config
        .call_json(conn, "mod_assign_get_assignments", process_assignments)
        .await?
    {
        insert_assignments(conn, &assigns)?;
    }

    Ok(())
}

async fn store_user(
    conn: &mut rusqlite::Connection,
    api_config: &mut ApiConfig,
) -> Result<(), CustomError> {
    api_config.userid = None;
    if let ProcessResult::User(user) = api_config
        .call_json(conn, "core_webservice_get_site_info", process_user)
        .await?
    {
        insert_user(conn, &user, api_config)?;
    }

    Ok(())
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
