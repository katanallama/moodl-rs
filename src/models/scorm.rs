// models/scorm.rs
use crate::db::{generic_insert, retrieve_param, Insertable, Retrievable};
use eyre::Result;
use rusqlite::{params, Connection, Row, ToSql};
use {serde::Deserialize, serde::Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Scorms {
    scorms: Vec<Scorm>,
    warnings: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Scorm {
    id: u64,
    coursemodule: u64,
    course: u64,
    name: String,
    intro: String,
    introfiles: Vec<serde_json::Value>,
    packageurl: String,
    version: String,
    maxgrade: u64,
    grademethod: u8,
    whatgrade: u8,
    maxattempt: u64,
}

pub fn insert_scorms(conn: &mut Connection, mut scorms: Scorms) -> Result<()> {
    let tx = conn.transaction()?;

    for scorm in scorms.scorms.iter_mut() {
        generic_insert(&tx, scorm)?;
    }

    tx.commit()?;
    log::info!("Sucessfully stored course scorms");
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
