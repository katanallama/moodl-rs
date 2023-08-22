// main.rs
//
mod models;
mod process_result;
use process_result::ProcessResult;

use clap::Parser;
use models::course::process_courses;
use models::course_grades::process_grades;
use models::recents::process_recents;
use models::response::{ApiParams, CustomError};
use reqwest;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    wstoken: String,
    #[arg(short, long)]
    courseid: i32,
    #[arg(short, long)]
    userid: i32,
}

struct User {
    id: i32,
}

async fn make_api_call(
    client: &reqwest::Client,
    url: &str,
    params: &ApiParams,
    process_fn: fn(&str) -> Result<ProcessResult, serde_json::Error>,
) -> Result<ProcessResult, CustomError> {
    let response_text = client.post(url).form(params).send().await?.text().await?;
    let result = process_fn(&response_text)?;
    Ok(result)
}

#[tokio::main]
async fn main() -> Result<(), CustomError> {
    let args = Cli::parse();
    let wstoken = &args.wstoken;
    let courseid = &args.courseid;
    let mut user: Option<User> = None;

    let client = reqwest::Client::new();
    let url = "https://urcourses.uregina.ca/webservice/rest/server.php";

    let params0 = ApiParams {
        wstoken: wstoken.clone(),
        wsfunction: "block_recentlyaccesseditems_get_recent_items".to_string(),
        moodlewsrestformat: "json".to_string(),
        courseid: None,
        userid: None,
        returnusercount: None,
    };

    let result = make_api_call(
        &client,
        url,
        &params0,
        process_recents as fn(&str) -> Result<ProcessResult, serde_json::Error>,
    )
    .await?;

    match result {
        ProcessResult::UserId(id) => {
            user = Some(User { id });
            println!("Stored User ID: {}", user.as_ref().unwrap().id);
        }
        ProcessResult::None => {}
    }

    let id = user.unwrap().id;

    let params1 = ApiParams {
        wstoken: wstoken.clone(),
        wsfunction: "core_enrol_get_users_courses".to_string(),
        moodlewsrestformat: "json".to_string(),
        courseid: None,
        userid: Some(id),
        returnusercount: Some(0),
    };

    let params2 = ApiParams {
        wstoken: wstoken.clone(),
        wsfunction: "gradereport_user_get_grades_table".to_string(),
        moodlewsrestformat: "json".to_string(),
        courseid: Some(courseid.clone()),
        // userid: Some(userid.clone()),
        userid: Some(id),
        returnusercount: None,
    };

    let result_courses = make_api_call(
        &client,
        url,
        &params1,
        process_courses as fn(&str) -> Result<ProcessResult, serde_json::Error>,
    )
    .await?;

    match result_courses {
        ProcessResult::UserId(_id) => {}
        ProcessResult::None => {}
    }

    let result_grades = make_api_call(
        &client,
        url,
        &params2,
        process_grades as fn(&str) -> Result<ProcessResult, serde_json::Error>,
    )
    .await?;

    match result_grades {
        ProcessResult::UserId(_id) => {}
        ProcessResult::None => {}
    }

    Ok(())
}

