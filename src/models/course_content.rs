// models/course_contents.rs
//
use crate::models::response::CustomError;
use crate::process_result::ProcessResult;
use chrono::Utc;
use rusqlite::{Connection, Result};
use serde_derive::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
pub struct CourseSection {
    pub id: i32,
    pub sectionid: Option<i64>,
    pub courseid: i32,
    pub name: String,
    pub modules: Vec<CourseModule>,
    pub summary: String,
    pub lastfetched: i64,
}

#[derive(Deserialize, Debug)]
pub struct CourseModule {
    pub id: i32,
    pub courseid: i32,
    pub moduleid: Option<i64>,
    pub modulename: String,
    pub content: String,
    pub lastfetched: i64,
}

#[derive(Deserialize, Debug)]
pub struct Assignment {
    pub id: i32,
    pub assignid: Option<i64>,
    pub courseid: Option<i64>,
    pub cmid: Option<i64>, // course module id
    pub content: String,
    pub lastfetched: i64,
}

#[derive(Deserialize, Debug)]
pub struct Grade {
    pub id: i32,
    pub gradeid: Option<i64>,
    pub courseid: Option<i64>,
    pub cmid: Option<i64>, // course module id
    pub content: String,
    pub lastfetched: i64,
}

pub fn process_grades(_conn: &Connection, content: &str) -> Result<ProcessResult, CustomError> {
    let parsed: Value = serde_json::from_str(content)?;

    let mut grade_list: Vec<Grade> = Vec::new();
    let now = Utc::now().timestamp();

    if let Some(course_grades) = parsed["usergrades"].as_array() {
        for course in course_grades {
            if let Some(grades) = course["gradeitems"].as_array() {
                for grade in grades {
                    let grd = Grade {
                        id: 0,
                        gradeid: grade["id"].as_i64(),
                        courseid: course_grades[0]["courseid"].as_i64(),
                        cmid: grade["cmid"].as_i64(),
                        content: grade.to_string(),
                        lastfetched: now,
                    };
                    grade_list.push(grd);
                }
            }
        }
    }

    Ok(ProcessResult::Grades(grade_list))
}

pub fn process_content(_conn: &Connection, content: &str) -> Result<ProcessResult, CustomError> {
    let parsed: Value = serde_json::from_str(content)?;
    let now = Utc::now().timestamp();

    let mut c_sections: Vec<CourseSection> = Vec::new();

    if let Some(sections) = parsed.as_array() {
        for section in sections {
            // println!("Section Name: {}", section["name"]);
            if let Some(modules) = section["modules"].as_array() {
                let mut c_modules: Vec<CourseModule> = Vec::new();

                for module in modules {
                    // println!("Module Name: {}", section["summary"]);
                    let c_module = CourseModule {
                        id: 0,
                        courseid: 0,
                        moduleid: module["id"].as_i64(),
                        modulename: module["name"].to_string(),
                        content: module.to_string(),
                        lastfetched: now,
                    };

                    c_modules.push(c_module);
                }

                let c_section = CourseSection {
                    id: 0,
                    courseid: 0,
                    sectionid: section["id"].as_i64(),
                    name: section["name"].to_string(),
                    modules: c_modules,
                    summary: section["summary"].to_string(),
                    lastfetched: now,
                };

                c_sections.push(c_section);
            }
        }
    }

    Ok(ProcessResult::Content(c_sections))
}

pub fn process_assignments(
    _conn: &Connection,
    content: &str,
) -> Result<ProcessResult, CustomError> {
    let parsed: Value = serde_json::from_str(content)?;
    let mut assignment_list: Vec<Assignment> = Vec::new();
    let now = Utc::now().timestamp();

    if let Some(courses) = parsed["courses"].as_array() {
        for course in courses {
            if let Some(assignments) = course["assignments"].as_array() {
                for assignment in assignments {
                    let assign = Assignment {
                        id: 0,
                        assignid: assignment["id"].as_i64(),
                        courseid: assignment["course"].as_i64(),
                        cmid: assignment["cmid"].as_i64(),
                        content: assignment.to_string(),
                        lastfetched: now,
                    };
                    assignment_list.push(assign);
                }
            }
        }
    }

    Ok(ProcessResult::Assigns(assignment_list))
}
