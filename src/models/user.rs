// models/user.rs
//
use crate::models::response::CustomError;
use crate::process_result::ProcessResult;
use chrono::Utc;
use rusqlite::{Connection, Result};
use serde_derive::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
pub struct User {
    pub id: i64,
    pub content: String,
    pub privkey: String,
    pub url: String,
    pub wstoken: String,
    pub lastfetched: i64,
}

pub fn process_user(_conn: &Connection, content: &str) -> Result<ProcessResult, CustomError> {
    let parsed: Value = serde_json::from_str(content)?;
    let now = Utc::now().timestamp();

    if let Some(user_id) = parsed["userid"].as_i64() {
        let user = User {
            id: user_id,
            content: parsed.to_string(),
            privkey: parsed["userprivateaccesskey"].to_string(),
            url: "".to_string(),
            wstoken: "".to_string(),
            lastfetched: now,
        };

        return Ok(ProcessResult::User(user));
    }

    Ok(ProcessResult::None)
}
