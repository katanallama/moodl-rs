// main.rs
mod db;
mod models;
mod process_result;
mod ws;

use clap::Parser;
use db::{
    create_courses_table, create_grades_table, get_grades, initialize_db, insert_course,
    insert_grade, insert_user,
};
use models::course::process_courses;
use models::course_grades::process_grades;
use models::recents::process_recents;
use models::response::CustomError;
use process_result::ProcessResult;
use reqwest;
use ws::ApiConfig;

#[derive(Parser)]
struct Cli {
    #[clap(short, long)]
    wstoken: Option<String>,
    #[clap(short, long)]
    courseid: Option<i32>,
}

#[tokio::main]
async fn main() -> Result<(), CustomError> {
    let args = Cli::parse();
    let conn = initialize_db()?;

    let mut api_config = ApiConfig {
        wstoken: args.wstoken,
        courseid: None,
        userid: None,
        client: reqwest::Client::new(),
        url: "https://urcourses.uregina.ca/webservice/rest/server.php",
    };

    // Get userid and create table
    if let ProcessResult::UserId(id) = api_config
        .call(
            "block_recentlyaccesseditems_get_recent_items",
            process_recents,
        )
        .await?
    {
        api_config.userid = Some(id);

        // Inser user data into db
        insert_user(
            &conn,
            id,
            api_config.wstoken.clone().expect("wstoken is missing!"),
            api_config.url.to_string(),
        )?;
    }

    // Course numbers to add
    let courses_to_add = vec!["353", "351"];

    // Get courses and create table
    if api_config.userid.is_some() {
        let result = api_config
            .call("core_enrol_get_users_courses", process_courses)
            .await?;

        // Insert course data into db
        if let ProcessResult::Courses(courses) = result {
            create_courses_table(&conn)?;
            create_grades_table(&conn)?;

            for course in &courses {
                if courses_to_add
                    .iter()
                    .any(|&num| course.shortname.contains(num))
                {
                    insert_course(&conn, course)?;

                    // Get grades/feedback for the course
                    api_config.courseid = Some(course.id);
                    let grades_result = api_config
                        .call("gradereport_user_get_grades_table", process_grades)
                        .await?;

                    if let ProcessResult::Grades(grades) = grades_result {
                        for grade in &grades {
                            insert_grade(&conn, grade)?;
                        }
                    }
                }
            }
        }
    }

    // Get grades/feedback for given courseid
    if let Some(course_id) = args.courseid {
        match get_grades(&conn, Some(course_id)) {
            Ok(grades) => {
                for (itemname, grade, feedback) in grades {
                    if let Some(name) = &itemname {
                        println!("Item Name: {}", name);
                    }

                    if let Some(g) = grade {
                        println!("Grade: {}", g);
                    }

                    if let Some(fb) = &feedback {
                        println!("Feedback: {}", fb);
                        println!("---------------------------");
                    }
                }
            }
            Err(e) => {
                println!("Error fetching grades: {}", e);
            }
        }
    }

    Ok(())
}
