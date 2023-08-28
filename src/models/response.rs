use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct Response {
    pub multiple: Option<Multiple>,
    pub single: Option<Vec<Single>>,
}

#[derive(Deserialize, Debug)]
pub struct Multiple {
    pub single: Vec<Single>,
}

#[derive(Deserialize, Debug)]
pub struct Single {
    pub key: Vec<Key>,
    pub _id: Option<String>,
    _idnumber: Option<String>,
    _displayname: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Key {
    pub name: String,
    pub value: Option<Value>,
}

#[derive(Deserialize, Debug)]
pub struct Value {
    pub value: Option<String>,
    _null: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiParams {
    pub courseid: Option<i32>,
    pub moodlewsrestformat: String,
    pub returnusercount: Option<i32>,
    pub userid: Option<i32>,
    pub wstoken: String,
    pub wsfunction: String,
}

#[derive(Debug)]
pub enum CustomError {
    Reqwest(reqwest::Error),
    SerdeJson(serde_json::Error),
    Rusqlite(rusqlite::Error),
    Io(std::io::Error),
    Api(String),
}

impl From<reqwest::Error> for CustomError {
    fn from(err: reqwest::Error) -> Self {
        CustomError::Reqwest(err)
    }
}

impl From<serde_json::Error> for CustomError {
    fn from(err: serde_json::Error) -> Self {
        CustomError::SerdeJson(err)
    }
}

impl From<rusqlite::Error> for CustomError {
    fn from(err: rusqlite::Error) -> Self {
        CustomError::Rusqlite(err)
    }
}

impl From<std::io::Error> for CustomError {
    fn from(err: std::io::Error) -> Self {
        CustomError::Io(err)
    }
}
