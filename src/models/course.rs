// models/course.rs
//
use crate::process_result::ProcessResult;
use scraper;
use serde_derive::Deserialize;
use serde_json;

#[derive(Deserialize, Debug)]
pub struct Course {
    pub id: i32,
    pub shortname: String,
    pub fullname: String,
    pub displayname: String,
    pub idnumber: String,
    pub visible: Option<i32>,
    pub summary: String,
    pub summaryformat: Option<i32>,
    pub format: Option<String>,
    pub showgrades: Option<bool>,
    pub lang: Option<String>,
    pub enablecompletion: Option<bool>,
    pub completionusertracked: Option<bool>,
    pub progress: Option<String>,
    pub startdate: Option<i64>,
    pub enddate: Option<i64>,
    pub marker: Option<i32>,
    pub lastaccess: Option<i64>,
    pub isfavourite: Option<bool>,
    pub hidden: Option<bool>,
    pub showactivitydates: Option<bool>,
    pub showcompletionconditions: Option<String>,
    pub timemodified: Option<i64>,
    pub category: Option<i32>,
    pub completed: Option<String>,
    pub completionhascriteria: Option<bool>,
}

fn sanitize_html(input: &str) -> String {
    let fragment = scraper::Html::parse_fragment(input);
    let text = fragment.root_element().text().collect::<String>();
    text.trim().to_string()
}

pub fn process_courses(response_text: &str) -> Result<ProcessResult, serde_json::Error> {
    let mut courses: Vec<Course> = serde_json::from_str(response_text)?;

    if !courses.is_empty() {
        for course in &mut courses {
            course.fullname = sanitize_html(&course.fullname);
            course.displayname = sanitize_html(&course.displayname);
            course.summary = sanitize_html(&course.summary);
        }
        return Ok(ProcessResult::Courses(courses));
    }

    Ok(ProcessResult::None)
}
