// ws.rs
#![allow(dead_code)]

use crate::models::{secrets::Secrets, course_section::Section, assignments::Assignments, courses::Course, grades::UserGrade, pages::Pages};
use anyhow::Result;
use {reqwest, serde::Deserialize, serde::Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct ApiConfig {
    base_url: String,
    token: String,
    userid: i64,
}

pub struct ApiClient {
    base_url: String,
    client: reqwest::Client,
    wstoken: String,
    userid: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApiResponse {
    Sections(Vec<Section>),
    Course(Vec<Course>),
    UserGrades(UserGradesResponse),
    Pages(Pages),
    Assignments(Assignments),
    Exception(ApiError),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserGradesResponse {
    usergrades: Vec<UserGrade>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    exception: String,
    errorcode: String,
    message: String,
    debuginfo: Option<String>,
}

#[derive(Serialize)]
pub struct QueryParameters<'a> {
    wsfunction: Option<String>,
    courseid: Option<i64>,
    userid: Option<i64>,
    moodlewsrestformat: String,
    wstoken: String,
    #[serde(skip)]
    client: &'a ApiClient,
}

impl<'a> QueryParameters<'a> {
    pub fn new(client: &'a ApiClient) -> Self {
        QueryParameters {
            wsfunction: None,
            courseid: None,
            userid: None,
            moodlewsrestformat: "json".to_string(),
            wstoken: "".to_string(),
            client,
        }
    }

    pub fn function(mut self, function: &str) -> Self {
        self.wsfunction = Some(function.to_string());
        self
    }

    pub fn courseid(mut self, courseid: i64) -> Self {
        self.courseid = Some(courseid);
        self
    }

    pub fn use_default_userid(mut self) -> Self {
        self.userid = Some(self.client.userid);
        self
    }

    pub fn userid(mut self, userid: Option<i64>) -> Self {
        self.userid = userid;
        self
    }
}


impl ApiClient {
    pub fn new(base_url: &str, token: &str, userid: &i64) -> Self {
        ApiClient {
            base_url: base_url.to_string(),
            wstoken: token.to_string(),
            client: reqwest::Client::new(),
            userid: userid.clone(),
        }
    }

    pub fn from_secrets(secrets: &Secrets) -> Result<Self> {
        Ok(ApiClient::new(
            &secrets.api.base_url,
            &secrets.api.token,
            &secrets.api.userid,
        ))
    }

    pub async fn _fetch_text<T: ApiQuery>(&self, query: T) -> Result<String> {
        let response = self
            .client
            .get(&self.base_url)
            .query(&query.with_token(&self.wstoken))
            .send()
            .await?;

        Ok(response.text().await?)
    }

    pub async fn fetch<T: ApiQuery>(&self, query: T) -> Result<ApiResponse> {
        let response = self
            .client
            .get(&self.base_url)
            .query(&query.with_token(&self.wstoken))
            .send()
            .await?;

        // println!("{:#?}", response);
        Ok(response.json::<ApiResponse>().await?)
    }
}

pub trait ApiQuery: Serialize {
    fn with_token(self, token: &str) -> Self;
    fn with_userid(self, userid: Option<i64>) -> Self;
}

impl ApiQuery for QueryParameters<'_> {
    fn with_token(mut self, token: &str) -> Self {
        self.wstoken = token.to_string();
        self
    }

    fn with_userid(mut self, userid: Option<i64>) -> Self {
        self.userid = userid;
        self
    }
}
