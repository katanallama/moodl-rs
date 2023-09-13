// models/course_details.rs
//
use eyre::Result;
use linked_hash_map::LinkedHashMap;
use rusqlite::{params, Connection};
use serde_derive::{Deserialize, Serialize};
use serde_json;

use crate::db::connect_db;

#[derive(Debug, Serialize, Deserialize)]
pub struct CourseDetails {
    pub section_name: Option<String>,
    pub section_summary: Option<String>,

    pub module_name: Option<String>,
    pub module_description: Option<String>,

    pub content_id: Option<i64>,
    pub content_filename: Option<String>,
    pub content_fileurl: Option<String>,
    pub content_localpath: Option<String>,

    pub page_name: Option<String>,
    pub page_intro: Option<String>,
    pub page_content: Option<String>,

    pub file_id: Option<i64>,
    pub file_filename: Option<String>,
    pub file_fileurl: Option<String>,
    pub file_localpath: Option<String>,
}

// When parsing we will go from
// CourseDetails to the following:
#[derive(Debug, Serialize, Deserialize)]
pub struct ParseCourseDetails {
    pub courseid: Option<i64>,
    pub sections: Vec<SectionDetails>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SectionDetails {
    pub section_name: Option<String>,
    pub section_summary: Option<String>,
    pub section_lastfetched: Option<String>,
    pub modules: Vec<ModuleDetails>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleDetails {
    pub module_name: Option<String>,
    pub module_description: Option<String>,
    pub module_instance: Option<i64>,
    pub module_contextid: Option<i64>,
    pub module_lastfetched: Option<String>,
    pub content: Vec<ContentDetails>,
    pub pages: Vec<PageDetails>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentDetails {
    pub content_id: Option<i64>,
    pub content_filename: Option<String>,
    pub content_fileurl: Option<String>,
    pub content_timemodified: Option<String>,
    pub content_lastfetched: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageDetails {
    pub page_name: Option<String>,
    pub page_intro: Option<String>,
    pub page_content: Option<String>,
    pub page_revision: Option<i64>,
    pub page_timemodified: Option<String>,
    pub page_lastfetched: Option<String>,
    pub files: Vec<FileDetails>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileDetails {
    pub file_id: Option<i64>,
    pub file_filename: Option<String>,
    pub file_fileurl: Option<String>,
    pub file_timemodified: Option<String>,
    pub file_lastfetched: Option<String>,
}

pub trait GetFileData {
    fn get_file_data(&self) -> Option<(String, String)>;
}

impl GetFileData for ContentDetails {
    fn get_file_data(&self) -> Option<(String, String)> {
        Some((
            self.content_filename.clone()?,
            self.content_fileurl.clone()?,
        ))
    }
}

impl GetFileData for FileDetails {
    fn get_file_data(&self) -> Option<(String, String)> {
        Some((self.file_filename.clone()?, self.file_fileurl.clone()?))
    }
}

pub fn get_course_details(conn: &Connection, course_id: i64) -> Result<Vec<CourseDetails>> {
    let mut stmt = conn.prepare(
        "
        SELECT
            Sections.id AS section_id,
            Sections.sectionid,
            Sections.name AS section_name,
            Sections.summary AS section_summary,
            Sections.lastfetched AS section_lastfetched,
            Sections.courseid,

            Modules.id AS module_id,
            Modules.moduleid,
            Modules.name AS module_name,
            Modules.instance AS module_instance,
            Modules.contextid AS module_contextid,
            Modules.description AS module_description,
            Modules.lastfetched AS module_lastfetched,

            Content.id AS content_id,
            Content.filename AS content_filename,
            Content.fileurl AS content_fileurl,
            Content.timemodified AS content_timemodified,
            Content.localpath AS content_localpath,
            Content.lastfetched AS content_lastfetched,

            Pages.id AS page_id,
            Pages.pageid,
            Pages.coursemodule,
            Pages.course,
            Pages.name AS page_name,
            Pages.intro AS page_intro,
            Pages.content AS page_content,
            Pages.revision AS page_revision,
            Pages.timemodified AS page_timemodified,
            Pages.lastfetched AS page_lastfetched,

            Files.id AS file_id,
            Files.filename AS file_filename,
            Files.fileurl AS file_fileurl,
            Files.timemodified AS file_timemodified,
            Files.localpath AS file_localpath,
            Files.lastfetched AS file_lastfetched

        FROM Sections

        LEFT JOIN Modules ON Sections.sectionid = Modules.section_id
        LEFT JOIN Content ON Modules.moduleid = Content.module_id
        LEFT JOIN Pages ON Modules.moduleid = Pages.coursemodule
        LEFT JOIN Files ON Pages.pageid = Files.page_id

        WHERE Sections.courseid = ?

        ORDER BY Sections.id, Modules.id, Content.id, Pages.id, Files.id
        ",
    )?;

    let course_details_iter = stmt.query_map(params![course_id], |row| {
        Ok(CourseDetails {
            // section_id: row.get("section_id")?,
            // sectionid: row.get("sectionid")?,
            section_name: row.get("section_name")?,
            section_summary: row.get("section_summary")?,
            // section_lastfetched: row.get("section_lastfetched")?,
            // courseid: row.get("courseid")?,

            // module_id: row.get("module_id")?,
            // moduleid: row.get("moduleid")?,
            module_name: row.get("module_name")?,
            // module_instance: row.get("module_instance")?,
            // module_contextid: row.get("module_contextid")?,
            module_description: row.get("module_description")?,
            // module_lastfetched: row.get("module_lastfetched")?,
            content_id: row.get("content_id")?,
            content_filename: row.get("content_filename")?,
            content_fileurl: row.get("content_fileurl")?,
            content_localpath: row.get("content_localpath")?,
            // content_timemodified: row.get("content_timemodified")?,
            // content_lastfetched: row.get("content_lastfetched")?,

            // page_id: row.get("page_id")?,
            // pageid: row.get("pageid")?,
            // coursemodule: row.get("coursemodule")?,
            // course: row.get("course")?,
            page_name: row.get("page_name")?,
            page_intro: row.get("page_intro")?,
            page_content: row.get("page_content")?,
            // page_revision: row.get("page_revision")?,
            // page_timemodified: row.get("page_timemodified")?,
            // page_lastfetched: row.get("page_lastfetched")?,
            file_id: row.get("file_id")?,
            file_filename: row.get("file_filename")?,
            file_fileurl: row.get("file_fileurl")?,
            file_localpath: row.get("file_localpath")?,
            // file_timemodified: row.get("file_timemodified")?,
            // file_lastfetched: row.get("file_lastfetched")?,
        })
    })?;

    let mut course_details = Vec::new();
    for course_detail_result in course_details_iter {
        course_details.push(course_detail_result?);
    }

    Ok(course_details)
}

pub fn parse_course_json(course_id: i64) -> Result<String> {
    let conn = connect_db();
    let course_details: Vec<CourseDetails> = get_course_details(&conn.unwrap(), course_id)?;

    if course_details.is_empty() {
        return Err(eyre::eyre!(
            "The 'Contents' table is empty, run 'Fetch' command first."
        ));
    }

    let mut section_map: LinkedHashMap<String, SectionDetails> = LinkedHashMap::new();

    for detail in course_details {
        let section_name = detail.section_name.unwrap_or_default();
        let module_name = detail.module_name.unwrap_or_default();

        let section = section_map
            .entry(section_name.clone())
            .or_insert_with(|| SectionDetails {
                section_name: Some(section_name.clone()),
                section_summary: detail.section_summary.clone(),
                section_lastfetched: None,
                modules: vec![],
            });

        let module_opt = section
            .modules
            .iter_mut()
            .find(|m| m.module_name.as_deref() == Some(module_name.as_str()));

        let module = if let Some(module) = module_opt {
            module
        } else {
            let new_module = ModuleDetails {
                module_name: Some(module_name.clone()),
                module_description: detail.module_description.clone(),
                module_instance: None,
                module_contextid: None,
                module_lastfetched: None,
                content: vec![],
                pages: vec![],
            };
            section.modules.push(new_module);
            section.modules.last_mut().unwrap()
        };

        // Add content details if not null
        if detail.content_filename.is_some() || detail.content_fileurl.is_some() {
            module.content.push(ContentDetails {
                content_id: detail.content_id.clone(),
                content_filename: detail.content_filename.clone(),
                content_fileurl: detail.content_fileurl.clone(),
                content_timemodified: None,
                content_lastfetched: None,
            });
        }

        // Add page details if not null
        if detail.page_name.is_some()
            || detail.page_intro.is_some()
            || detail.page_content.is_some()
        {
            let mut new_page = PageDetails {
                page_name: detail.page_name.clone(),
                page_intro: detail.page_intro.clone(),
                page_content: detail.page_content.clone(),
                page_revision: None,
                page_timemodified: None,
                page_lastfetched: None,
                files: vec![FileDetails {
                    file_id: detail.file_id.clone(),
                    file_filename: detail.file_filename.clone(),
                    file_fileurl: detail.file_filename.clone(),
                    file_timemodified: None,
                    file_lastfetched: None,
                }],
            };

            // Add file details if not null
            if detail.file_filename.is_some() || detail.file_fileurl.is_some() {
                new_page.files.push(FileDetails {
                    file_id: detail.file_id.clone(),
                    file_filename: detail.file_filename.clone(),
                    file_fileurl: detail.file_filename.clone(),
                    file_timemodified: None,
                    file_lastfetched: None,
                });
            }

            module.pages.push(new_page);
        }
    }

    let parsed_course_details = ParseCourseDetails {
        courseid: Some(course_id),
        sections: section_map.into_iter().map(|(_, v)| v).collect(),
    };

    let json_data = serde_json::to_string_pretty(&parsed_course_details)?;

    Ok(json_data)
}
