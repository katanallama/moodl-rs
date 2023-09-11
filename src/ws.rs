// ws.rs
//
use crate::models::{
    assignments::Assignments, configs::Configs, course_section::Section, courses::Course,
    grades::UserGrade, pages::Pages, user::SiteInfo,
};
use eyre::Result;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest;
use serde::{Deserialize, Serialize};
use std::{cmp::min, fs::File, io::Write};

pub struct ApiClient {
    base_url: String,
    client: reqwest::Client,
    wstoken: String,
    userid: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApiResponse {
    Exception(ApiError),
    SiteInfo(SiteInfo),
    Sections(Vec<Section>),
    Course(Vec<Course>),
    UserGrades(UserGradesResponse),
    Pages(Pages),
    Assignments(Assignments),
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
        log::info!("New API Client created");
        ApiClient {
            base_url: base_url.to_string(),
            wstoken: token.to_string(),
            client: reqwest::Client::new(),
            userid: userid.clone(),
        }
    }

    pub fn from_config(configs: &Configs) -> Result<Self> {
        log::info!("Using API config from file");
        log::debug!("Base url from config: {}", configs.api.base_url);
        log::debug!("Token from config: {}", configs.api.token);
        log::debug!("UserID from config: {}", configs.api.userid);
        Ok(ApiClient::new(
            &configs.api.base_url,
            &configs.api.token,
            &configs.api.userid,
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

        let response_text = response.text().await?;

        // First, try to parse the response as an ApiError
        if let Ok(api_error) = serde_json::from_str::<ApiError>(&response_text) {
            return Err(eyre::eyre!("API Error: {:?}", api_error));
        }

        // If parsing as ApiError failed, try to parse it as an ApiResponse
        match serde_json::from_str::<ApiResponse>(&response_text) {
            Ok(api_response) => Ok(api_response),
            Err(_) => Err(eyre::eyre!("Failed to parse API response")),
        }
    }

    pub async fn download_file(&self, url: &str, file_path: &str) -> Result<(), eyre::Report> {
        let url_with_token = format!("{}&token={}", url, self.wstoken);

        let res = self
            .client
            .get(&url_with_token)
            .send()
            .await
            .map_err(|_| eyre::eyre!("Failed to GET from '{}'", &url))?;

        let total_size = res
            .content_length()
            .ok_or_else(|| eyre::eyre!("Failed to get content length from '{}'", &url))?;

        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{wide_bar:.cyan/blue}] {bytes}/{total_bytes}")
                .progress_chars("#>-"),
        );

        let mut file = File::create(file_path)
            .map_err(|_| eyre::eyre!("Failed to create file '{}'", file_path))?;

        let mut downloaded: u64 = 0;
        let mut stream = res.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item.map_err(|_| eyre::eyre!("Error while downloading file"))?;
            file.write_all(&chunk)
                .map_err(|_| eyre::eyre!("Error while writing to file"))?;
            let new = min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            pb.set_position(new);
        }

        pb.finish();
        log::info!("Downloaded file {:?}", file_path);
        Ok(())
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
