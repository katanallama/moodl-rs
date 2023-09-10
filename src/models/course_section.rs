// models/course_section.rs
//
use crate::db::generic_insert;
use crate::db::Insertable;
use eyre::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Section {
    pub id: i64,
    pub name: String,
    pub summary: String,
    pub courseid: Option<i64>,
    pub modules: Vec<Module>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Module {
    pub id: i64,
    pub name: String,
    pub instance: Option<i64>,
    pub contextid: Option<i64>,
    pub description: Option<String>,
    pub contents: Option<Vec<Content>>,
    pub section_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    #[serde(rename = "type")]
    pub content_type: String,
    pub filename: Option<String>,
    pub fileurl: Option<String>,
    pub timemodified: Option<i64>,
    pub module_id: Option<i64>,
}

pub fn insert_sections(
    conn: &mut rusqlite::Connection,
    sections: &mut [Section],
    courseid: i64,
) -> Result<()> {
    let tx = conn.transaction()?;

    for section in sections.iter_mut() {
        section.courseid = Some(courseid);
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

impl Insertable for Section {
    fn insert_query() -> &'static str {
        "INSERT INTO Sections (sectionid, name, summary, courseid, lastfetched)
            VALUES (:sectionid, :name, :summary, :courseid, CURRENT_TIMESTAMP)
            ON CONFLICT(sectionid) DO UPDATE SET
                name=excluded.name,
                summary=excluded.summary,
                lastfetched=excluded.lastfetched"
    }

    fn bind_parameters(&self) -> Vec<(&'static str, &dyn rusqlite::ToSql)> {
        vec![
            (":sectionid", &self.id),
            (":name", &self.name),
            (":summary", &self.summary),
            (":courseid", &self.courseid),
        ]
    }
}

impl Insertable for Module {
    fn insert_query() -> &'static str {
        "INSERT INTO Modules (moduleid, name, instance, contextid, description, section_id, lastfetched)
            VALUES (:moduleid, :name, :instance, :contextid, :description, :section_id, CURRENT_TIMESTAMP)
            ON CONFLICT(moduleid) DO UPDATE SET
                name=excluded.name,
                instance=excluded.instance,
                contextid=excluded.contextid,
                description=excluded.description,
                section_id=excluded.section_id,
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
        ]
    }
}

impl Insertable for Content {
    fn insert_query() -> &'static str {
        "INSERT INTO Content (filename, fileurl, timemodified, module_id, lastfetched)
            VALUES (:filename, :fileurl, :timemodified, :module_id, CURRENT_TIMESTAMP)
            ON CONFLICT(filename, fileurl, module_id) DO UPDATE SET
                timemodified=excluded.timemodified,
                lastfetched=excluded.lastfetched"
    }

    fn bind_parameters(&self) -> Vec<(&'static str, &dyn rusqlite::ToSql)> {
        vec![
            (":filename", &self.filename),
            (":fileurl", &self.fileurl),
            (":timemodified", &self.timemodified),
            (":module_id", &self.module_id),
        ]
    }
}
