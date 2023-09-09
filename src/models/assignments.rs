use {serde::Deserialize, serde::Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Assignments {
    pub courses: Vec<Course>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Course {
    pub id: u32,
    pub timemodified: u64,
    pub assignments: Vec<Assignment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Assignment {
    pub id: u32,
    pub cmid: u32,
    pub course: u32,
    pub name: String,
    pub duedate: u64,
    pub allowsubmissionsfromdate: u64,
    // pub grade: Option<i32>,
    pub timemodified: u64,
    pub cutoffdate: u64,
    pub intro: Option<String>,
    // pub introfiles: Vec<String>, // This might be a different type depending on the data structure
    pub introattachments: Option<Vec<IntroAttachment>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntroAttachment {
    pub filename: String,
    pub filepath: String,
    pub filesize: u64,
    pub fileurl: String,
    pub timemodified: u64,
}
