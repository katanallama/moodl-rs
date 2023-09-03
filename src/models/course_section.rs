use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Section {
    id: i32,
    name: Option<String>,
    summary: Option<String>,
    section: Option<i32>,
    modules: Option<Vec<Module>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Module {
    id: i32,
    url: Option<String>,
    name: String,
    instance: Option<i32>,
    contextid: Option<i32>,
    description: Option<String>,
    // modname: Option<String>,
    // availability: Option<String>,
    // onclick: Option<String>,
    // customdata: Option<String>,
    // completion: Option<i32>,
    completiondata: Option<CompletionData>,
    // downloadcontent: Option<i32>,
    dates: Option<Vec<Date>>,
    contents: Option<Vec<Content>>,
    contentsinfo: Option<ContentsInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionData {
    state: i32,
    timecompleted: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Date {
    label: String,
    timestamp: i32,
    relativeto: Option<i32>,
    dataid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    #[serde(rename = "type")]
    content_type: String,
    filename: Option<String>,
    filepath: Option<String>,
    fileurl: Option<String>,
    content: Option<String>,
    timecreated: Option<i32>,
    timemodified: Option<i32>,
    repositorytype: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentsInfo {
    filescount: Option<i32>,
    lastmodified: Option<i32>,
}
