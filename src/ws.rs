// ws.rs
//
use crate::db::get_user;
use crate::models::response::{ApiParams, CustomError};
use crate::process_result::ProcessResult;
use serde_derive::Deserialize;
use std::io::{self};

pub struct ApiConfig {
    pub client: reqwest::Client,
    pub courseid: Option<i32>,
    pub url: String,
    pub userid: Option<i32>,
    pub wstoken: String,
}

#[derive(Deserialize, Debug)]
pub struct ApiError {
    exception: String,
    errorcode: String,
    message: String,
    _debuginfo: Option<String>,
}

impl ApiConfig {
    pub async fn call_json(
        &self,
        conn: &rusqlite::Connection,
        wsfunction: &str,
        process_fn: fn(&rusqlite::Connection, &str) -> Result<ProcessResult, CustomError>,
    ) -> Result<ProcessResult, CustomError> {
        let params = ApiParams {
            wstoken: self.wstoken.clone(),
            wsfunction: wsfunction.to_string(),
            moodlewsrestformat: "json".to_string(),
            courseid: self.courseid,
            userid: self.userid,
            returnusercount: None,
        };

        let response_text = self
            .client
            .post(self.url.clone())
            .form(&params)
            .send()
            .await?
            .text()
            .await?;

        if let Ok(api_error) = serde_json::from_str::<ApiError>(&response_text) {
            match &api_error.exception[..] {
                "moodle_exception" => {
                    if api_error.errorcode == "sitemaintenance" {
                        return Err(CustomError::Api(format!(
                            "Exception: {}. Message: {}",
                            api_error.exception, api_error.message
                        )));
                    }
                }
                "required_capability_exception" => {
                    if api_error.errorcode == "nopermissions" {
                        return Err(CustomError::Api(format!(
                            "Exception: {}. Message: {}",
                            api_error.exception, api_error.message
                        )));
                    }
                }
                "invalid_parameter_exception" => {
                    if api_error.errorcode == "invalidparameter" {
                        return Err(CustomError::Api(format!(
                            "Exception: {}. Message: {}",
                            api_error.exception, api_error.message,
                        )));
                    }
                }
                _ => {
                    // TODO General error handling
                }
            }
        }

        Ok(process_fn(&conn, &response_text)?)
    }

    pub fn get_saved_api_config(conn: &rusqlite::Connection) -> Result<Self, CustomError> {
        match get_user(&conn, None) {
            Ok(Some((userid, wstoken, url))) => Ok(ApiConfig {
                wstoken,
                courseid: None,
                userid: Some(userid),
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
