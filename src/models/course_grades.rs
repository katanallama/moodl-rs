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
    pub courseid: u32,
    pub userid: u32,
    pub userfullname: String,
    pub maxdepth: u8,
    pub tabledata: Vec<TableData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableData {
    pub itemname: Option<ItemDetails>,
    pub leader: Option<ItemDetails>,
    pub weight: Option<ItemDetails>,
    pub grade: Option<ItemDetails>,
    pub range: Option<ItemDetails>,
    pub feedback: Option<ItemDetails>,
    pub contributiontocoursetotal: Option<ItemDetails>,
    parentcategories: Option<Vec<u32>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemDetails {
    class: String,
    colspan: Option<u8>,
    pub content: Option<String>,
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
    let response: ApiResponse = serde_json::from_str(response_text)?;

    let mut processed_tables = Vec::new();

    for mut table in response.tables {
        for data in &mut table.tabledata {
            if let Some(item_details) = &mut data.itemname {
                if let Some(content) = &mut item_details.content {
                    *content = sanitize_html(content);
                }
            }
            if let Some(item_details) = &mut data.grade {
                if let Some(content) = &mut item_details.content {
                    *content = sanitize_html(content);
                }
            }
            if let Some(item_details) = &mut data.feedback {
                if let Some(content) = &mut item_details.content {
                    *content = sanitize_html(content);
                }
            }
        }
        processed_tables.push(table);
    }

    Ok(ProcessResult::Grades(processed_tables))
}
