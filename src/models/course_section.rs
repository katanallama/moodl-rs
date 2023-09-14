// models/course_section.rs
//
use crate::db::{generic_insert, Insertable};
use eyre::Result;
use html2md::parse_html;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Section {
    pub id: i64,
    pub name: String,
    pub summary: String,
    pub courseid: Option<i64>,
    pub timemodified: Option<i64>,
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
    pub timemodified: Option<i64>,
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

impl Section {
    pub fn process(&mut self) {
        if let desc = &mut self.summary {
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

            let name = &mut self.name;
            if desc.contains(&name.to_string()) {
                println!("{}", desc);
            }

            let _ = name.remove_matches("\r");
            let _ = name.remove_matches("\n");
            let _ = name.remove_matches("<br>");

            let _ = desc.remove_matches(&name.to_string());
            let _ = desc.remove_matches("<h4></h4>");
            let _ = desc.remove_matches("<h5></h5>");
            *desc = parse_html(&desc);
        }
    }
}

impl Module {
    pub fn process(&mut self) {
        if let Some(desc) = &mut self.description {
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

            let name = &mut self.name;
            if desc.contains(&name.to_string()) {
                println!("{}", desc);
            }

            let _ = name.remove_matches("\r");
            let _ = name.remove_matches("\n");
            let _ = name.remove_matches("<br>");

            let _ = desc.remove_matches(&name.to_string());
            let _ = desc.remove_matches("<h4></h4>");
            let _ = desc.remove_matches("<h5></h5>");
            *desc = parse_html(&desc);
        }
    }
}

pub fn insert_sections(
    conn: &mut rusqlite::Connection,
    sections: &mut [Section],
    courseid: i64,
) -> Result<()> {
    let tx = conn.transaction()?;

    for section in sections.iter_mut() {
        section.courseid = Some(courseid);
        section.process();
        generic_insert(&tx, section)?;

        for module in section.modules.iter_mut() {
            module.process();
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

impl Insertable for Module {
    fn insert_query() -> &'static str {
        "INSERT INTO Modules (moduleid, name, instance, contextid, description, section_id, timemodified, lastfetched)
            VALUES (:moduleid, :name, :instance, :contextid, :description, :section_id, :timemodified, CURRENT_TIMESTAMP)
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
