// models/grades.rs
use {serde::Deserialize, serde::Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserGrade {
    courseid: u32,
    userid: u32,
    gradeitems: Vec<GradeItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GradeItem {
    id: u32,
    itemname: Option<String>,
    itemmodule: Option<String>,
    iteminstance: u32,
    itemnumber: Option<u32>,
    idnumber: Option<String>,
    categoryid: Option<u32>,
    cmid: Option<u32>,
    graderaw: Option<String>,
    gradedatesubmitted: Option<String>,
    gradedategraded: Option<String>,
    grademin: u32,
    grademax: u32,
    feedback: String,
}
