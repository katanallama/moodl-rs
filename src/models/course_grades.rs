// models/course_grades.rs
//
use crate::process_result::ProcessResult;

use scraper;
use serde_derive::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    tables: Vec<Table>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Table {
    courseid: u32,
    userid: u32,
    userfullname: String,
    maxdepth: u8,
    tabledata: Vec<TableData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableData {
    itemname: Option<ItemDetails>,
    leader: Option<ItemDetails>,
    weight: Option<ItemDetails>,
    grade: Option<ItemDetails>,
    range: Option<ItemDetails>,
    feedback: Option<ItemDetails>,
    contributiontocoursetotal: Option<ItemDetails>,
    parentcategories: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemDetails {
    #[serde(rename = "class")]
    class_name: String,
    colspan: Option<u8>,
    content: Option<String>,
    id: Option<String>,
    rowspan: Option<u8>,
    headers: Option<String>,
}

fn sanitize_html(input: &str) -> String {
    let fragment = scraper::Html::parse_fragment(input);
    let text = fragment.root_element().text().collect::<String>();
    text.trim().to_string()
}

pub fn process_grades(response_text: &str) -> Result<ProcessResult, serde_json::Error> {
    // Deserialize the response into ApiResponse
    let response: ApiResponse = serde_json::from_str(response_text)?;

    // Use the deserialized data
    for table in response.tables {
        println!("Course ID: {}", table.courseid);
        println!("User ID: {}", table.userid);
        println!("User Full Name: {}", table.userfullname);
        println!("---------------------------");

        for data in table.tabledata {
            if let Some(item_details) = &data.itemname {
                if let Some(ref content) = item_details.content {
                    let sanitized_name = sanitize_html(content);
                    println!("Name: {}", sanitized_name);
                }
            }
            if let Some(item_details) = &data.grade {
                if let Some(ref content) = item_details.content {
                    let sanitized_grade = sanitize_html(content);
                    println!("Grade: {}", sanitized_grade);
                }
            }
            if let Some(item_details) = &data.feedback {
                if let Some(ref content) = item_details.content {
                    let sanitized_feedback = sanitize_html(content);
                    println!("Feedback: {}", sanitized_feedback);
                }
            }
            println!("---------------------------");
        }
    }

    Ok(ProcessResult::None)
}
