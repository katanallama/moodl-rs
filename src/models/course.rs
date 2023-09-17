// models/course.rs
//
use crate::db::retrieve_param;
use crate::db::{generic_insert, generic_retrieve, Insertable, Retrievable};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rusqlite::{Connection, ToSql, Row, params};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CourseSection {
    pub id: i64,
    pub name: String,
    pub summary: String,
    pub courseid: Option<i64>,
    pub timemodified: Option<i64>,
    pub modules: Vec<CourseModule>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CourseModule {
    pub id: i64,
    pub name: String,
    pub instance: Option<i64>,
    pub contextid: Option<i64>,
    pub description: Option<String>,
    pub contents: Option<Vec<CourseFile>>,
    pub timemodified: Option<i64>,
    pub section_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CourseFile {
    pub filename: Option<String>,
    pub filepath: Option<String>,
    pub fileurl: Option<String>,
    pub timemodified: Option<i64>,
    pub module_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pages {
    pub pages: Vec<Page>,
    pub warnings: Vec<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    pub id: i64,
    pub coursemodule: i64,
    pub course: i64,
    pub name: Option<String>,
    pub intro: Option<String>,
    pub introfiles: Vec<CourseFile>,
    pub content: Option<String>,
    pub contentfiles: Vec<CourseFile>,
    pub revision: i64,
    pub timemodified: Option<i64>,
}

impl CourseSection {
    pub fn combine(&mut self, pages_root: &Pages) {
        for page in &pages_root.pages {
            for module in self.modules.iter_mut() {
                if let Some(page_content) = &page.content {
                    if module.id == page.coursemodule {
                        module.description = Some(page_content.clone());

                        let mut files = page.contentfiles.clone();
                        let mut intro_files = page.introfiles.clone();
                        files.append(&mut intro_files);

                        module.contents = Some(files);
                    }
                }
            }
        }
    }
}

pub fn insert_course_sections(
    conn: &mut Connection,
    sections: &mut Vec<CourseSection>,
    pages: &Pages,
    courseid: i64,
) -> Result<()> {
    let tx = conn.transaction()?;

    for section in sections.iter_mut() {
        section.courseid = Some(courseid);
        section.combine(pages);
        generic_insert(&tx, section)?;

        for module in section.modules.iter_mut() {
            module.section_id = Some(section.id);
            generic_insert(&tx, module)?;

            if let Some(contents) = &mut module.contents {
                for content in contents.iter_mut() {
                    content.module_id = Some(module.id);
                    generic_insert(&tx, content)?;
                }
            }
        }
    }

    tx.commit()?;
    Ok(())
}

impl Insertable for CourseSection {
    fn insert_query() -> &'static str {
        "INSERT INTO Sections (sectionid, name, summary, courseid, timemodified, lastfetched)
            VALUES (:sectionid, :name, :summary, :courseid, :timemodified, CURRENT_TIMESTAMP)
            ON CONFLICT(sectionid) DO UPDATE SET
                name=excluded.name,
                summary=excluded.summary,
                timemodified=excluded.timemodified,
                lastfetched=excluded.lastfetched"
    }

    fn bind_parameters(&self) -> Vec<(&'static str, &dyn ToSql)> {
        vec![
            (":sectionid", &self.id),
            (":name", &self.name),
            (":summary", &self.summary),
            (":courseid", &self.courseid),
            (":timemodified", &self.timemodified),
        ]
    }
}

impl Insertable for CourseModule {
    fn insert_query() -> &'static str {
        "INSERT INTO Modules (moduleid, name, instance, contextid, description, content, section_id, timemodified, lastfetched)
            VALUES (:moduleid, :name, :instance, :contextid, :description, :content, :section_id, :timemodified, CURRENT_TIMESTAMP)
            ON CONFLICT(moduleid) DO UPDATE SET
                name=excluded.name,
                instance=excluded.instance,
                contextid=excluded.contextid,
                description=excluded.description,
                section_id=excluded.section_id,
                timemodified=excluded.timemodified,
                lastfetched=excluded.lastfetched"
    }

    fn bind_parameters(&self) -> Vec<(&'static str, &dyn ToSql)> {
        vec![
            (":moduleid", &self.id),
            (":name", &self.name),
            (":instance", &self.instance),
            (":contextid", &self.contextid),
            (":description", &self.description),
            (":section_id", &self.section_id),
            (":timemodified", &self.timemodified),
        ]
    }
}

impl Insertable for CourseFile {
    fn insert_query() -> &'static str {
        "INSERT INTO Files (filename, fileurl, timemodified, module_id, lastfetched)
            VALUES (:filename, :fileurl, :timemodified, :module_id,  CURRENT_TIMESTAMP)
            ON CONFLICT(filename) DO UPDATE SET
                timemodified=excluded.timemodified,
                lastfetched=excluded.lastfetched"
    }

    fn bind_parameters(&self) -> Vec<(&'static str, &dyn ToSql)> {
        vec![
            (":filename", &self.filename),
            (":fileurl", &self.fileurl),
            (":filepath", &self.filepath),
            (":timemodified", &self.timemodified),
            (":module_id", &self.module_id),
        ]
    }
}

impl Retrievable for CourseSection {
    fn select_query() -> &'static str {
        "SELECT sectionid, name, summary, courseid, timemodified
            FROM Sections WHERE courseid = ?1"
    }
    fn select_query_all() -> &'static str {
        "SELECT sectionid, name, summary, courseid, timemodified
            FROM Sections"
    }

    fn from_row(row: &Row) -> Result<Self> {
        Ok(CourseSection {
            id: row.get("sectionid")?,
            name: row.get("name")?,
            summary: row.get("summary")?,
            courseid: row.get("courseid")?,
            timemodified: row.get("timemodified")?,
            modules: Vec::new(), // empty vector
        })
    }
}

impl Retrievable for CourseModule {
    fn select_query() -> &'static str {
        "SELECT moduleid, name, instance, contextid, description, timemodified, section_id
            FROM Modules WHERE section_id = ?1"
    }

    fn select_query_all() -> &'static str {
        "SELECT moduleid, name, instance, contextid, description, timemodified, section_id
            FROM Modules"
    }

    fn from_row(row: &Row) -> Result<Self> {
        Ok(CourseModule {
            id: row.get("moduleid")?,
            name: row.get("name")?,
            instance: row.get("instance")?,
            contextid: row.get("contextid")?,
            description: row.get("description")?,
            contents: Some(Vec::new()), // empty vector
            timemodified: row.get("timemodified")?,
            section_id: row.get("section_id")?,
        })
    }
}

impl Retrievable for CourseFile {
    fn select_query() -> &'static str {
        "SELECT filename, fileurl, localpath, timemodified, module_id
            FROM Files WHERE module_id = ?1"
    }

    fn select_query_all() -> &'static str {
        "SELECT filename, fileurl, localpath, timemodified, module_id
            FROM Files"
    }

    fn from_row(row: &Row) -> Result<Self> {
        Ok(CourseFile {
            filename: row.get("filename")?,
            fileurl: row.get("fileurl")?,
            filepath: row.get("localpath")?,
            timemodified: row.get("timemodified")?,
            module_id: row.get("module_id")?,
        })
    }
}

pub fn retrieve_course_structure(
    conn: &mut Connection,
    courseid: i64
) -> Result<Vec<CourseSection>> {
    log::info!("Retrieving course structure");

    let tx = conn.transaction().map_err(|e| {
        log::error!("Failed to start transaction: {:?}", e);
        e
    })?;

    log::info!("Transaction started");

    let mut sections: Vec<CourseSection> = retrieve_param(&tx, params![courseid])?;

    for section in sections.iter_mut() {

        let mut modules: Vec<CourseModule> =
            retrieve_param(&tx, params![section.id])?;

        for module in modules.iter_mut() {
            let files: Vec<CourseFile> = retrieve_param(&tx, params![module.id])?;
            if !files.is_empty() {
                module.contents = Some(files);
            }
        }
        section.modules = modules.clone();
    }

    tx.commit().map_err(|e| {
        log::error!("Failed to commit transaction: {:?}", e);
        e
    })?;

    Ok(sections)
}

pub fn get_all_files(conn: &mut Connection) -> Result<Vec<CourseFile>> {
    let tx = conn.transaction()?;
    let files = generic_retrieve(&tx)?;

    tx.commit()?;
    Ok(files)
}
