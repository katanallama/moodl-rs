// ws.rs
//
use crate::db::get_user;
use crate::models::response::{ApiParams, CustomError};
use crate::process_result::ProcessResult;

use std::io::{self};

pub struct ApiConfig {
    pub client: reqwest::Client,
    pub courseid: Option<i32>,
    pub url: String,
    pub userid: Option<i32>,
    pub wstoken: String,
}

impl ApiConfig {
    pub async fn call(
        &self,
        wsfunction: &str,
        process_fn: fn(&str) -> Result<ProcessResult, serde_json::Error>,
    ) -> Result<ProcessResult, CustomError> {
        let params = ApiParams {
            wstoken: self.wstoken.clone(),
            wsfunction: wsfunction.to_string(),
            moodlewsrestformat: "json".to_string(),
            courseid: self.courseid,
            userid: self.userid,
            returnusercount: if wsfunction == "core_enrol_get_users_courses" {
                Some(0)
            } else {
                None
            },
        };

        let response_text = self
            .client
            .post(self.url.clone())
            .form(&params)
            .send()
            .await?
            .text()
            .await?;

        Ok(process_fn(&response_text)?)
    }

    pub fn get_saved_api_config(conn: &rusqlite::Connection) -> Result<Self, CustomError> {
        match get_user(&conn, None) {
            Ok(Some((_, wstoken, url))) => Ok(ApiConfig {
                wstoken,
                courseid: None,
                userid: None,
                client: reqwest::Client::new(),
                url,
            }),
            Ok(None) => Err(CustomError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "User not found in database. Please run the init command.",
            ))),
            Err(e) => Err(CustomError::Rusqlite(e)),
        }
    }
}
