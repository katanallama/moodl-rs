// models/grades.rs
//
use {serde::Deserialize, serde::Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserGrade {
    courseid: i64,
    userid: i64,
    gradeitems: Vec<GradeItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GradeItem {
    id: i64,
    itemname: Option<String>,
    // itemmodule: Option<String>,
    iteminstance: i64,
    itemnumber: Option<i64>,
    // idnumber: Option<String>,
    categoryid: Option<i64>,
    cmid: Option<i64>,
    graderaw: Option<i64>,
    gradedatesubmitted: Option<i64>,
    gradedategraded: Option<i64>,
    grademin: Option<i64>,
    grademax: Option<i64>,
    feedback: Option<String>,
}
