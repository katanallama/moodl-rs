mod models;

use clap::Parser;
use models::course::process_courses;
use models::course_grades::process_grades;
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

async fn make_api_call(
    client: &reqwest::Client,
    url: &str,
    params: &ApiParams,
    process_fn: fn(&str) -> Result<(), serde_json::Error>,
) -> Result<(), CustomError> {
    let response_text = client.post(url).form(params).send().await?.text().await?;

    process_fn(&response_text)?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), CustomError> {
    let args = Cli::parse();
    let wstoken = &args.wstoken;
    let courseid = &args.courseid;
    let userid = &args.userid;

    let client = reqwest::Client::new();
    let url = "https://urcourses.uregina.ca/webservice/rest/server.php";

    let params = ApiParams {
        wstoken: wstoken.clone(),
        wsfunction: "core_enrol_get_users_courses".to_string(),
        moodlewsrestformat: "json".to_string(),
        courseid: None,
        userid: Some(userid.clone()),
        returnusercount: Some(0),
    };

    let params2 = ApiParams {
        wstoken: wstoken.clone(),
        wsfunction: "gradereport_user_get_grades_table".to_string(),
        moodlewsrestformat: "json".to_string(),
        courseid: Some(courseid.clone()),
        userid: Some(userid.clone()),
        returnusercount: None,
    };

    // Call for courses
    make_api_call(&client, url, &params, process_courses).await?;

    // Call for grades
    make_api_call(&client, url, &params2, process_grades).await?;

    Ok(())
}

