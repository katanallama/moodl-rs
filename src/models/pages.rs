// models/pages.rs
//
use crate::db::generic_insert;
use crate::db::Insertable;
use anyhow::Result;
use rusqlite::ToSql;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Pages {
    pub pages: Vec<Page>,
    warnings: Vec<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    id: i64,
    coursemodule: i64,
    course: i64,
    name: Option<String>,
    intro: Option<String>,
    introfiles: Vec<File>,
    content: Option<String>,
    contentfiles: Vec<File>,
    revision: i64,
    timemodified: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    filename: Option<String>,
    filepath: Option<String>,
    fileurl: Option<String>,
    timemodified: Option<i64>,
    page_id: Option<i64>,
}

pub fn insert_pages(conn: &mut rusqlite::Connection, pages: &mut [Page]) -> Result<()> {
    let tx = conn.transaction()?;

    for page in pages.iter_mut() {
        generic_insert(&tx, page)?;

        for introfile in page.introfiles.iter_mut() {
            introfile.page_id = Some(page.id);
            generic_insert(&tx, introfile)?;
        }

        for contentfile in page.contentfiles.iter_mut() {
            contentfile.page_id = Some(page.id);
            generic_insert(&tx, contentfile)?;
        }
    }

    tx.commit()?;
    Ok(())
}

impl Insertable for Page {
    fn insert_query() -> &'static str {
        "INSERT INTO Pages (pageid, coursemodule, course, name, intro, content, revision, timemodified, lastfetched)
            VALUES (:pageid, :coursemodule, :course, :name, :intro, :content, :revision, :timemodified, CURRENT_TIMESTAMP)
            ON CONFLICT(pageid) DO UPDATE SET
                coursemodule=excluded.coursemodule,
                course=excluded.course,
                name=excluded.name,
                intro=excluded.intro,
                content=excluded.content,
                revision=excluded.revision,
                timemodified=excluded.timemodified,
                lastfetched=excluded.lastfetched"
    }

    fn bind_parameters(&self) -> Vec<(&'static str, &dyn ToSql)> {
        vec![
            (":pageid", &self.id),
            (":coursemodule", &self.coursemodule),
            (":course", &self.course),
            (":name", &self.name),
            (":intro", &self.intro),
            (":content", &self.content),
            (":revision", &self.revision),
            (":timemodified", &self.timemodified),
        ]
    }
}

impl Insertable for File {
    fn insert_query() -> &'static str {
        "INSERT INTO Files (filename, fileurl, timemodified, page_id, lastfetched)
            VALUES (:filename, :fileurl, :timemodified, :page_id, CURRENT_TIMESTAMP)
            ON CONFLICT(filename, page_id) DO UPDATE SET
                fileurl=excluded.fileurl,
                timemodified=excluded.timemodified,
                lastfetched=excluded.lastfetched"
    }

    fn bind_parameters(&self) -> Vec<(&'static str, &dyn ToSql)> {
        vec![
            (":filename", &self.filename),
            (":fileurl", &self.fileurl),
            (":timemodified", &self.timemodified),
            (":page_id", &self.page_id),
        ]
    }
}
