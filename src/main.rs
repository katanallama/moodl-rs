// main.rs
mod models;
mod process_result;
mod ws;

use clap::Parser;
use models::course::process_courses;
use models::course_grades::process_grades;
use models::recents::process_recents;
use models::response::CustomError;
use process_result::ProcessResult;
use reqwest;
use ws::api_config::ApiConfig;

#[derive(Parser)]
struct Cli {
    #[clap(short, long)]
    wstoken: String,
    #[clap(short, long)]
    courseid: Option<i32>,
}

#[tokio::main]
async fn main() -> Result<(), CustomError> {
    let args = Cli::parse();

    let mut api_config = ApiConfig {
        wstoken: args.wstoken,
        courseid: None,
        userid: None,
        client: reqwest::Client::new(),
        url: "https://urcourses.uregina.ca/webservice/rest/server.php",
    };

    if let ProcessResult::UserId(id) = api_config
        .call(
            "block_recentlyaccesseditems_get_recent_items",
            process_recents,
        )
        .await?
    {
        println!("Stored User ID: {}", id);
        api_config.userid = Some(id);
    }

    if api_config.userid.is_some() {
        api_config.call("core_enrol_get_users_courses", process_courses).await?;

        if let Some(course_id) = args.courseid {
            api_config.courseid = Some(course_id);
            api_config
                .call("gradereport_user_get_grades_table", process_grades)
                .await?;
        }
    }

    Ok(())
}
