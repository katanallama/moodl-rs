use std::collections::HashMap;
use {serde::Deserialize, serde::Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Pages {
    pages: Vec<Page>,
    warnings: Vec<HashMap<String, String>>,  // Assuming warnings contain key-value pairs
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    id: u32,
    coursemodule: u32,
    course: u32,
    name: String,
    intro: String,
    introfiles: Vec<File>,  // Or Vec<HashMap<String, String>> if the structure is unknown
    section: u8,
    // visible: bool,
    // groupmode: u8,
    // groupingid: u32,
    content: String,
    contentfiles: Vec<File>,
    // legacyfiles: u8,
    // legacyfileslast: Option<u32>,
    // display: u8,
    // displayoptions: String,
    revision: u32,
    timemodified: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    filename: String,
    filepath: String,
    fileurl: String,
    timemodified: u64,
}
