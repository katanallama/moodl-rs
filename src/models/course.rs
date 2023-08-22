// models/course.rs
//
use crate::process_result::ProcessResult;
use serde_derive::Deserialize;
use serde_json;

#[derive(Deserialize, Debug)]
pub struct Course {
    pub id: i32,
    pub shortname: String,
    pub fullname: String,
    pub displayname: String,
    pub idnumber: String,
    pub visible: i32,
    pub summary: String,
    pub summaryformat: i32,
    pub format: String,
    pub showgrades: bool,
    pub lang: String,
    pub enablecompletion: bool,
    pub completionhascriteria: bool,
    pub completionusertracked: bool,
    pub category: i32,
    pub progress: Option<String>,
    pub completed: Option<String>,
    pub startdate: i64,
    pub enddate: i64,
    pub marker: i32,
    pub lastaccess: i64,
    pub isfavourite: bool,
    pub hidden: bool,
    pub overviewfiles: Vec<OverviewFile>,
    pub showactivitydates: bool,
    pub showcompletionconditions: Option<String>,
    pub timemodified: i64,
}

#[derive(Deserialize, Debug)]
pub struct OverviewFile {
    pub filename: String,
    pub filepath: String,
    pub filesize: i32,
    pub fileurl: String,
    pub timemodified: i64,
    pub mimetype: String,
}

pub fn process_courses(response_text: &str) -> Result<ProcessResult, serde_json::Error> {
    // Deserialize the response into a Vec<Course>
    let courses: Vec<Course> = serde_json::from_str(response_text)?;

    // Use the deserialized data
    for course in courses {
        println!(
            "ID :\t{}\nName :\t{}\n---------------------------",
            course.id, course.shortname
        );
    }

    Ok(ProcessResult::None)
}
