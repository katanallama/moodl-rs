// models/assignments.rs
//
use crate::db::{generic_insert, retrieve_param, Insertable, Retrievable};
use eyre::Result;
use log;
use rusqlite::{params, Connection, Row, ToSql};
use serde::{Deserialize, Serialize};

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
    pub courseid: Option<i64>,
}

pub fn insert_assignments(conn: &mut Connection, mut assignments: Assignments) -> Result<()> {
    let tx = conn.transaction()?;

    for course in assignments.courses.iter_mut() {
        for assign in course.assignments.iter_mut() {
            assign.courseid = Some(course.id);
            generic_insert(&tx, assign)?;
        }
    }

    tx.commit()?;
    log::info!("Successfully stored assignments");
    Ok(())
}

impl Insertable for Assignment {
    fn insert_query() -> &'static str {
        "INSERT INTO Assignments (
        assignid, cmid, course, name, duedate, submissionsopen, timemodified, cutoffdate, intro, courseid, lastfetched)
            VALUES (
                :assignid, :cmid, :course, :name, :duedate, :submissionsopen, :timemodified, :cutoffdate, :intro, :courseid, CURRENT_TIMESTAMP
            )
            ON CONFLICT(assignid) DO UPDATE SET
                cmid=excluded.cmid,
                course=excluded.course,
                name=excluded.name,
                duedate=excluded.duedate,
                submissionsopen=excluded.submissionsopen,
                timemodified=excluded.timemodified,
                cutoffdate=excluded.cutoffdate,
                intro=excluded.intro,
                courseid=excluded.courseid,
                lastfetched=excluded.lastfetched"
    }

    fn bind_parameters(&self) -> Vec<(&'static str, &dyn ToSql)> {
        vec![
            (":assignid", &self.id),
            (":cmid", &self.cmid),
            (":course", &self.course),
            (":name", &self.name),
            (":duedate", &self.duedate),
            (":submissionsopen", &self.allowsubmissionsfromdate),
            (":timemodified", &self.timemodified),
            (":cutoffdate", &self.cutoffdate),
            (":intro", &self.intro),
            (":courseid", &self.courseid),
        ]
    }
}

impl Retrievable for Assignment {
    fn select_query() -> &'static str {
        "SELECT id, cmid, course, name, duedate, submissionsopen, timemodified, cutoffdate, intro, courseid
            FROM Assignments WHERE course = ?1"
    }

    fn select_query_all() -> &'static str {
        "SELECT id, cmid, course, name, duedate, submissionsopen, timemodified, cutoffdate, intro, courseid
            FROM Assignments"
    }

    fn from_row(row: &Row) -> Result<Self> {
        Ok(Assignment {
            id: row.get("id")?,
            cmid: row.get("cmid")?,
            course: row.get("course")?,
            // iteminstance: row.get("course")?,
            name: row.get("name")?,
            duedate: row.get("duedate")?,
            allowsubmissionsfromdate: row.get("submissionsopen")?,
            timemodified: row.get("timemodified")?,
            cutoffdate: row.get("cutoffdate")?,
            intro: row.get("intro")?,
            courseid: row.get("courseid")?,
        })
    }
}

pub fn retrieve_course_assignments(
    conn: &mut Connection,
    courseid: i64,
) -> Result<Vec<Assignment>> {
    log::debug!("Retrieving course {} assignments", courseid);
    let tx = conn.transaction()?;
    let assignments: Vec<Assignment> = retrieve_param(&tx, params![courseid])?;
    tx.commit()?;
    log::info!("Successfully retrieved course {} assignments", courseid);
    Ok(assignments)
}
