// ws.rs
//
#![allow(dead_code)]

use crate::models::{
    assignments::Assignments, course_section::Section, courses::Course, grades::UserGrade,
    pages::Pages, configs::Configs, user::SiteInfo
};
use anyhow::Result;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::cmp::min;
use std::fs::File;
use std::io::Write;
use {reqwest, serde::Deserialize, serde::Serialize};

pub struct ApiClient {
    base_url: String,
    client: reqwest::Client,
    wstoken: String,
    userid: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApiResponse {
    SiteInfo(SiteInfo),
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

    pub fn from_config(configs: &Configs) -> Result<Self> {
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

        // println!("{:#?}", response);
        Ok(response.json::<ApiResponse>().await?)
    }

    // TODO error handling
    pub async fn download_file(&self, url: &str, file_path: &str) -> Result<(), anyhow::Error> {
    // pub async fn download_file(&self, url: &str, file_path: &str) -> Result<(), String> {
        let url_with_token = format!("{}&token={}", url, self.wstoken);

        // Reqwest setup
        let res = self.client
            .get(&url_with_token)
            .send()
            .await
            // .or(Err(format!("Failed to GET from '{}'", &url)))?;
            .map_err(|_| anyhow::anyhow!("Failed to GET from '{}'", &url))?;

        let total_size = res
            .content_length()
            // .ok_or(format!("Failed to get content length from '{}'", &url))?;
            .ok_or_else(|| anyhow::anyhow!("Failed to get content length from '{}'", &url))?;

        // Indicatif setup
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
            .progress_chars("#>-"));
        pb.set_message(&format!("\nDownloading {}", url));

        // Download chunks
        let mut file =
            // File::create(file_path).or(Err(format!("Failed to create file '{}'", file_path)))?;
            File::create(file_path).map_err(|_| anyhow::anyhow!("Failed to create file '{}'", file_path))?;

        let mut downloaded: u64 = 0;
        let mut stream = res.bytes_stream();

        while let Some(item) = stream.next().await {
            // let chunk = item.or(Err(format!("Error while downloading file")))?;
            let chunk = item.map_err(|_| anyhow::anyhow!("Error while downloading file"))?;
            file.write_all(&chunk)
                // .or(Err(format!("Error while writing to file")))?;
                .map_err(|_| anyhow::anyhow!("Error while writing to file"))?;
            let new = min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            pb.set_position(new);
        }

        pb.finish_with_message(&format!("Downloaded to {}", file_path));
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
