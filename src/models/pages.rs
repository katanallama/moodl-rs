// models/pages.rs
//
use crate::db::{generic_insert, Insertable};
use eyre::Result;
use html2md::parse_html;
use rusqlite::ToSql;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub introfiles: Vec<File>,
    pub content: Option<String>,
    pub contentfiles: Vec<File>,
    pub revision: i64,
    pub timemodified: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub filename: Option<String>,
    pub filepath: Option<String>,
    pub fileurl: Option<String>,
    pub timemodified: Option<i64>,
    pub page_id: Option<i64>,
}

impl Page {
    pub fn process(&mut self) {
        if let Some(desc) = &mut self.content {
            let _ = desc.remove_matches(" dir=\"ltr\"");
            let _ = desc.remove_matches(" style=\"text-align: left;\"");
            let _ = desc.remove_matches("<p></p>");
            let _ = desc.remove_matches("<br>");
            let _ = desc.remove_matches("<div class=\"no-overflow\">");
            let _ = desc.remove_matches("</div>");
            let _ = desc.remove_matches("<span lang=\"EN-US\">");
            let _ = desc.remove_matches("</span>");
            let _ = desc.remove_matches("\r");
            let _ = desc.remove_matches("\n");

            // let name = &mut self.name;
            // if desc.contains(&name.to_string()) {
            //     println!("{}", desc);
            // }


            // let _ = desc.remove_matches(&name.to_string());
            let _ = desc.remove_matches("<h4></h4>");
            let _ = desc.remove_matches("<h5></h5>");
            *desc = parse_html(&desc);
        }
    }
}

pub fn insert_pages(conn: &mut rusqlite::Connection, pages: &mut [Page]) -> Result<()> {
    let tx = conn.transaction()?;

    for page in pages.iter_mut() {
        page.process();
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
