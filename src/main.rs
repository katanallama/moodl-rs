// main.rs
mod db;
mod models;
mod process_result;
mod ws;

use clap::Parser;
use db::{
    create_course_content_tables, create_courses_table, create_grades_table, create_user_table,
    initialize_db, insert_assignments, insert_content, insert_course, insert_grade, insert_user,
};
use models::course::process_courses;
use models::course_content::process_assignments;
use models::course_content::process_content;
use models::course_grades::process_grades;
use models::recents::process_recents;
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
        init(&conn)?
    } else {
        ApiConfig::get_saved_api_config(&conn)?
    };

    process_user(&conn, &mut api_config).await?;
    process_courses_to_add(&conn, &mut api_config).await?;
    store_course_content(&mut conn, &mut api_config, 28962).await?;
    store_course_content(&mut conn, &mut api_config, 26490).await?;

    Ok(())
}

async fn store_course_content(
    conn: &mut rusqlite::Connection,
    api_config: &mut ApiConfig,
    course_id: i32,
) -> Result<(), CustomError> {
    create_course_content_tables(conn)?;
    api_config.courseid = Some(course_id);
    api_config.userid = None;
    if let ProcessResult::Content(cont) = api_config
        .call_json(conn, "core_course_get_contents", process_content)
        .await?
    {
        insert_content(conn, api_config.courseid, &cont)?;
    }

    api_config.courseid = None;

    if let ProcessResult::Assigns(assigns) = api_config
        .call_json(conn, "mod_assign_get_assignments", process_assignments)
        .await?
    {
        insert_assignments(conn, &assigns)?;
    }

    Ok(())
}

fn init(conn: &rusqlite::Connection) -> Result<ApiConfig, CustomError> {
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

async fn process_user(
    conn: &rusqlite::Connection,
    api_config: &mut ApiConfig,
) -> Result<(), CustomError> {
    if let ProcessResult::UserId(id) = api_config
        .call(
            "block_recentlyaccesseditems_get_recent_items",
            process_recents,
        )
        .await?
    {
        api_config.userid = Some(id);
        insert_user(
            conn,
            id,
            api_config.wstoken.clone(),
            api_config.url.to_string(),
        )?;
    }
    Ok(())
}

async fn process_courses_to_add(
    conn: &rusqlite::Connection,
    api_config: &mut ApiConfig,
) -> Result<(), CustomError> {
    let courses_to_add = vec!["353", "351", "452", "472", "303"];
    if api_config.userid.is_some() {
        let result = api_config
            .call("core_enrol_get_users_courses", process_courses)
            .await?;

        if let ProcessResult::Courses(courses) = result {
            create_courses_table(conn)?;
            create_grades_table(conn)?;

            for course in &courses {
                if courses_to_add
                    .iter()
                    .any(|&num| course.shortname.contains(num))
                {
                    insert_course(conn, course)?;

                    api_config.courseid = Some(course.id);
                    let grades_result = api_config
                        // .call("gradereport_user_get_grades_items", process_grades) // TODO use this instead
                        .call("gradereport_user_get_grades_table", process_grades)
                        .await?;

                    if let ProcessResult::Grades(grades) = grades_result {
                        for grade in &grades {
                            insert_grade(conn, grade)?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

// fn display_grades_for_course(
//     conn: &rusqlite::Connection,
//     courseid: Option<i32>,
// ) -> Result<(), CustomError> {
//     if let Some(course_id) = courseid {
//         match get_grades(conn, Some(course_id)) {
//             Ok(grades) => {
//                 for (itemname, grade, feedback) in grades {
//                     if let Some(name) = &itemname {
//                         if let Some(g) = grade {
//                             println!("{}\t|  {}", g, name);
//                         }
//                     }

//                     if let Some(fb) = &feedback {
//                         println!("Feedback: {}", fb);
//                         println!("------------------------------------------------------");
//                     }
//                 }
//             }
//             Err(e) => {
//                 println!("Error fetching grades: {}", e);
//             }
//         }
//     }
//     Ok(())
// }
