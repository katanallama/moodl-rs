// models/pages.rs
//
use std::collections::HashMap;
use {serde::Deserialize, serde::Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Pages {
    pages: Vec<Page>,
    warnings: Vec<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    id: i64,
    coursemodule: i64,
    course: i64,
    name: Option<String>,
    intro: Option<String>,
    introfiles: Vec<File>,
    content: Option<String>,
    contentfiles: Vec<File>,
    revision: i64,
    timemodified: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    filename: Option<String>,
    filepath: Option<String>,
    fileurl: Option<String>,
    timemodified: Option<i64>,
    page_id: Option<i64>,
}
