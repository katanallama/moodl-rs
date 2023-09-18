use {serde::Deserialize, serde::Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Assignments {
    pub courses: Vec<Course>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Course {
    pub id: i64,
    pub timemodified: i64,
    pub assignments: Vec<Assignment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Assignment {
    pub id: i64,
    pub cmid: i64,
    pub course: i64,
    pub name: String,
    pub duedate: i64,
    pub allowsubmissionsfromdate: i64,
    pub timemodified: i64,
    pub cutoffdate: i64,
    pub intro: Option<String>,
    pub introfiles: Vec<File>,
    pub introattachments: Option<Vec<File>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub filename: String,
    pub filepath: String,
    pub filesize: i64,
    pub fileurl: String,
    pub timemodified: i64,
}
