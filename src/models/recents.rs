// models/recents.rs
//
use crate::process_result::ProcessResult;
use serde_derive::Deserialize;
use serde_json;

#[derive(Deserialize, Debug)]
pub struct Recent {
    pub id: i32,
    pub courseid: i32,
    pub cmid: i32,
    pub userid: i32,
    pub modname: String,
    pub name: String,
    pub coursename: String,
    pub timeaccess: i32,
    pub viewurl: String,
    pub courseviewurl: String,
    pub icon: String,
    pub purpose: String,
}

pub fn process_recents(response_text: &str) -> Result<ProcessResult, serde_json::Error> {
    let recents: Vec<Recent> = serde_json::from_str(response_text)?;

    for recent in recents {
        return Ok(ProcessResult::UserId(recent.userid));
    }

    Ok(ProcessResult::None)
}
