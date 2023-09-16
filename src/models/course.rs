// models/course.rs
//
use crate::db::{generic_insert, generic_retrieve, Insertable, Retrievable};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    conn: &mut rusqlite::Connection,
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

    fn bind_parameters(&self) -> Vec<(&'static str, &dyn rusqlite::ToSql)> {
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

    fn bind_parameters(&self) -> Vec<(&'static str, &dyn rusqlite::ToSql)> {
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

    fn bind_parameters(&self) -> Vec<(&'static str, &dyn rusqlite::ToSql)> {
        vec![
            (":filename", &self.filename),
            (":fileurl", &self.fileurl),
            (":filepath", &self.filepath),
            (":timemodified", &self.timemodified),
            (":module_id", &self.module_id),
        ]
    }
}

impl Retrievable for CourseFile {
    fn select_query() -> &'static str {
        "SELECT filename, fileurl, localpath, timemodified, module_id FROM Files"
    }

    fn from_row(row: &rusqlite::Row) -> Result<Self> {
        Ok(CourseFile {
            filename: row.get(0)?,
            fileurl: row.get(1)?,
            filepath: row.get(2)?,
            timemodified: row.get(3)?,
            module_id: row.get(4)?,
        })
    }
}

pub fn get_all_files(conn: &mut rusqlite::Connection) -> Result<Vec<CourseFile>> {
    let tx = conn.transaction()?;
    let files = generic_retrieve(&tx)?;

    tx.commit()?;
    Ok(files)
}
