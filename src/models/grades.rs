// models/grades.rs
//
use crate::db::{generic_insert, retrieve_param, Insertable, Retrievable};
use eyre::Result;
use rusqlite::{params, Connection, Row, ToSql};
use {serde::Deserialize, serde::Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CourseGrades {
    courseid: i64,
    userid: i64,
    pub gradeitems: Vec<GradeItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GradeItem {
    pub id: i64,
    pub itemname: Option<String>,
    pub itemmodule: Option<String>,
    pub iteminstance: i64,
    pub itemnumber: Option<i64>,
    pub idnumber: Option<String>,
    pub categoryid: Option<i64>,
    pub cmid: Option<i64>,
    pub graderaw: Option<i64>,
    pub gradedatesubmitted: Option<i64>,
    pub gradedategraded: Option<i64>,
    pub grademin: i64,
    pub grademax: i64,
    pub feedback: Option<String>,
    pub courseid: Option<i64>,
}

pub fn insert_grades(conn: &mut Connection, mut course_grades: Vec<CourseGrades>) -> Result<()> {
    let tx = conn.transaction()?;

    for grades in course_grades.iter_mut() {
        for grade in grades.gradeitems.iter_mut() {
            grade.courseid = Some(grades.courseid);
            generic_insert(&tx, grade)?;
        }
    }

    tx.commit()?;
    log::info!("Sucessfully stored course grades");
    Ok(())
}

impl Insertable for GradeItem {
    fn insert_query() -> &'static str {
        "INSERT INTO Grades (
        gradeid, itemname, itemmodule, iteminstance, itemnumber, idnumber, categoryid,
        cmid, graderaw, gradedatesubmitted, gradedategraded, grademin, grademax, feedback,
        courseid, lastfetched)
            VALUES (
                :gradeid, :itemname, :itemmodule, :iteminstance, :itemnumber, :idnumber, :categoryid,
                :cmid, :graderaw, :gradedatesubmitted, :gradedategraded, :grademin, :grademax, :feedback,
                :courseid, CURRENT_TIMESTAMP
            )
            ON CONFLICT(gradeid) DO UPDATE SET
                itemname=excluded.itemname,
                itemmodule=excluded.itemmodule,
                iteminstance=excluded.iteminstance,
                itemnumber=excluded.itemnumber,
                idnumber=excluded.idnumber,
                categoryid=excluded.categoryid,
                cmid=excluded.cmid,
                graderaw=excluded.graderaw,
                gradedatesubmitted=excluded.gradedatesubmitted,
                gradedategraded=excluded.gradedategraded,
                grademin=excluded.grademin,
                grademax=excluded.grademax,
                feedback=excluded.feedback,
                courseid=excluded.courseid,
                lastfetched=excluded.lastfetched"
    }

    fn bind_parameters(&self) -> Vec<(&'static str, &dyn ToSql)> {
        log::debug!("Binding parameters for GradeItem");
        log::debug!("gradeid: {}", &self.id);
        log::debug!("itemname: {:?}", &self.itemname);
        log::debug!("itemmodule: {:?}", &self.itemmodule);
        log::debug!("iteminstance: {:?}", &self.iteminstance);
        log::debug!("itemnumber: {:?}", &self.itemnumber);
        log::debug!("idnumber: {:?}", &self.idnumber);
        log::debug!("categoryid: {:?}", &self.categoryid);
        log::debug!("cmid: {:?}", &self.cmid);
        log::debug!("graderaw: {:?}", &self.graderaw);
        log::debug!("gradedatesubmitted: {:?}", &self.gradedatesubmitted);
        log::debug!("gradedategraded: {:?}", &self.gradedategraded);
        log::debug!("grademin: {:?}", &self.grademin);
        log::debug!("grademax: {:?}", &self.grademax);
        log::debug!("feedback: {:?}", &self.feedback);
        log::debug!("courseid: {:?}", &self.courseid);

        vec![
            (":gradeid", &self.id),
            (":itemname", &self.itemname),
            (":itemmodule", &self.itemmodule),
            (":iteminstance", &self.iteminstance),
            (":itemnumber", &self.itemnumber),
            (":idnumber", &self.idnumber),
            (":categoryid", &self.categoryid),
            (":cmid", &self.cmid),
            (":graderaw", &self.graderaw),
            (":gradedatesubmitted", &self.gradedatesubmitted),
            (":gradedategraded", &self.gradedategraded),
            (":grademin", &self.grademin),
            (":grademax", &self.grademax),
            (":feedback", &self.feedback),
            (":courseid", &self.courseid),
        ]
    }
}

impl Retrievable for GradeItem {
    fn select_query() -> &'static str {
        "SELECT gradeid, itemname, itemmodule, iteminstance, itemnumber, idnumber, categoryid, cmid,
            graderaw, gradedatesubmitted, gradedategraded, grademin, grademax, feedback, courseid
            FROM Grades WHERE courseid = ?1"
    }
    fn select_query_all() -> &'static str {
        "SELECT gradeid, itemname, itemmodule, iteminstance, itemnumber, idnumber, categoryid, cmid,
            graderaw, gradedatesubmitted, gradedategraded, grademin, grademax, feedback, courseid
            FROM Grades"
    }

    fn from_row(row: &Row) -> Result<Self> {
        Ok(GradeItem {
            id: row.get("gradeid")?,
            itemname: row.get("itemname")?,
            itemmodule: row.get("itemmodule")?,
            iteminstance: row.get("iteminstance")?,
            itemnumber: row.get("itemnumber")?,
            idnumber: row.get("idnumber")?,
            categoryid: row.get("categoryid")?,
            cmid: row.get("cmid")?,
            graderaw: row.get("graderaw")?,
            gradedatesubmitted: row.get("gradedatesubmitted")?,
            gradedategraded: row.get("gradedategraded")?,
            grademin: row.get("grademin")?,
            grademax: row.get("grademax")?,
            feedback: row.get("feedback")?,
            courseid: row.get("courseid")?,
        })
    }
}

pub fn retrieve_course_grades(conn: &mut Connection, courseid: i64) -> Result<Vec<GradeItem>> {
    log::debug!("Retrieving course {} grades", courseid);
    let tx = conn.transaction().map_err(|e| {
        log::error!("Failed to start transaction: {:?}", e);
        e
    })?;

    log::debug!("Transaction started");

    let grades: Vec<GradeItem> = retrieve_param(&tx, params![courseid])?;

    tx.commit().map_err(|e| {
        log::error!("Failed to commit transaction: {:?}", e);
        e
    })?;

    log::info!("Successfully retrieved course {} grades", courseid);
    Ok(grades)
}
