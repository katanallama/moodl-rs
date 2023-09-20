// models/scorm.rs
//
use crate::db::{generic_insert, Insertable, Retrievable};
use crate::models::course::CourseFile;
use eyre::Result;
use rusqlite::{Connection, Row, ToSql};
use {serde::Deserialize, serde::Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Scorms {
    pub scorms: Vec<Scorm>,
    warnings: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Scorm {
    pub id: i64,
    pub coursemodule: i64,
    pub course: i64,
    pub name: String,
    pub intro: String,
    pub introfiles: Vec<serde_json::Value>,
    pub packageurl: String,
    pub version: String,
    pub maxgrade: i64,
    pub grademethod: u8,
    pub whatgrade: u8,
    pub maxattempt: i64,
}

fn remove_emojis(filename: String) -> String {
    filename.chars().filter(|&c| c.is_ascii()).collect()
}

pub fn insert_scorms(conn: &mut Connection, mut scorms: Scorms) -> Result<()> {
    let tx = conn.transaction()?;

    for scorm in scorms.scorms.iter_mut() {
        match &scorm.packageurl {
            url => {
                let file_name = remove_emojis(scorm.name.clone());
                let file_url = &format!("{}?forcedownload=1", url);
                let file = CourseFile {
                    filename: Some(file_name),
                    filepath: None,
                    fileurl: Some(file_url.to_string()),
                    timemodified: None,
                    module_id: Some(scorm.coursemodule.clone()),
                };
                generic_insert(&tx, &file)?;
            }
            // _ => (),
        }
        generic_insert(&tx, scorm)?;
    }

    tx.commit()?;
    log::info!("Successfully stored course scorms");
    Ok(())
}

impl Insertable for Scorm {
    fn insert_query() -> &'static str {
        "INSERT INTO Scorms (
        scormid, coursemodule, courseid, name, intro, packageurl, version,
        maxgrade, grademethod, whatgrade, maxattempt, lastfetched)
        VALUES (:scormid, :coursemodule, :courseid, :name, :intro, :packageurl, :version,
            :maxgrade, :grademethod, :whatgrade, :maxattempt, CURRENT_TIMESTAMP)
        ON CONFLICT(scormid) DO UPDATE SET
            coursemodule=excluded.coursemodule,
            courseid=excluded.courseid,
            name=excluded.name,
            intro=excluded.intro,
            packageurl=excluded.packageurl,
            version=excluded.version,
            maxgrade=excluded.maxgrade,
            grademethod=excluded.grademethod,
            whatgrade=excluded.whatgrade,
            maxattempt=excluded.maxattempt,
            lastfetched=excluded.lastfetched"
    }

    fn bind_parameters(&self) -> Vec<(&'static str, &dyn ToSql)> {
        log::debug!("Binding parameters for GradeItem");
        log::debug!("scormid: {}", &self.id);
        log::debug!("coursemodule: {:?}", &self.coursemodule);
        log::debug!("courseid: {:?}", &self.course);
        log::debug!("name: {:?}", &self.name);
        log::debug!("intro: {:?}", &self.intro);
        log::debug!("introfiles: {:?}", &self.introfiles);
        log::debug!("packageurl: {:?}", &self.packageurl);
        log::debug!("version: {:?}", &self.version);
        log::debug!("maxgrade: {:?}", &self.maxgrade);
        log::debug!("grademethod: {:?}", &self.grademethod);
        log::debug!("whatgrade: {:?}", &self.whatgrade);
        log::debug!("maxattempt: {:?}", &self.maxattempt);

        vec![
            (":scormid", &self.id),
            (":coursemodule", &self.coursemodule),
            (":courseid", &self.course),
            (":name", &self.name),
            (":intro", &self.intro),
            (":packageurl", &self.packageurl),
            (":version", &self.version),
            (":maxgrade", &self.maxgrade),
            (":grademethod", &self.grademethod),
            (":whatgrade", &self.whatgrade),
            (":maxattempt", &self.maxattempt),
        ]
    }
}

impl Retrievable for Scorm {
    fn select_query() -> &'static str {
        "SELECT scormid, coursemodule, courseid, name, intro, packageurl, version,
            maxgrade, grademethod, whatgrade, maxattempt, lastfetched
            FROM Scorms WHERE courseid = ?1"
    }

    fn select_query_all() -> &'static str {
        "SELECT scormid, coursemodule, courseid, name, intro, packageurl, version,
            maxgrade, grademethod, whatgrade, maxattempt, lastfetched
            FROM Scorms"
    }

    fn from_row(row: &Row) -> Result<Self> {
        Ok(Scorm {
            id: row.get("scormid")?,
            coursemodule: row.get("coursemodule")?,
            course: row.get("courseid")?,
            name: row.get("name")?,
            intro: row.get("intro")?,
            introfiles: Vec::new(), // empty vector
            packageurl: row.get("packageurl")?,
            version: row.get("version")?,
            maxgrade: row.get("maxgrade")?,
            grademethod: row.get("grademethod")?,
            whatgrade: row.get("whatgrade")?,
            maxattempt: row.get("maxattempt")?,
        })
    }
}
