use crate::models::course_section::{Content, Module, Section};
use crate::models::pages::{File, Page, Pages};
use anyhow::Result;
use rusqlite::{params, Connection};
use serde::Serialize;
use serde_json;
use std::fs::File as StdFile;
use std::io::Write;

pub fn write_course_content_to_file(
    conn: &Connection,
    course_id: i64,
    file_path: &str,
) -> Result<()> {
    let pages = get_pages_by_course_id(conn, course_id)?;
    let sections = get_sections_by_course_id(conn, course_id)?;

    #[derive(Serialize)]
    struct CourseContent {
        pages: Pages,
        sections: Vec<Section>,
    }

    let course_content = CourseContent { pages, sections };

    let json_data = serde_json::to_string_pretty(&course_content)?;

    let mut file = StdFile::create(file_path)?;
    file.write_all(json_data.as_bytes())?;

    Ok(())
}

pub fn get_pages_by_course_id(conn: &Connection, course_id: i64) -> Result<Pages> {
    let mut stmt = conn.prepare("SELECT * FROM Pages WHERE course = ?")?;
    let pages_iter = stmt.query_map(params![course_id], |row| {
        Ok(Page {
            id: row.get(1)?,
            coursemodule: row.get(2)?,
            course: row.get(3)?,
            name: row.get(4)?,
            intro: row.get(5)?,
            introfiles: get_files_by_page_id(conn, row.get(0)?)?,
            // content: row.get(7),
            content: None,
            contentfiles: get_files_by_page_id(conn, row.get(0)?)?,
            revision: row.get(7)?,
            // timemodified: row.get(10)?,
            timemodified: None,
        })
    })?;

    let mut pages = Vec::new();
    for page_result in pages_iter {
        pages.push(page_result?);
    }

    Ok(Pages {
        pages,
        warnings: vec![],
    })
}

fn get_files_by_page_id(conn: &Connection, page_id: i64) -> rusqlite::Result<Vec<File>> {
    let mut stmt = conn.prepare("SELECT * FROM Files WHERE page_id = ?")?;
    let files_iter = stmt.query_map(params![page_id], |row| {
        Ok(File {
            filename: row.get(0)?,
            filepath: row.get(1)?,
            fileurl: row.get(2)?,
            timemodified: row.get(3)?,
            page_id: row.get(4)?,
        })
    })?;

    let mut files = Vec::new();
    for file_result in files_iter {
        files.push(file_result?);
    }

    Ok(files)
}

fn get_modules_by_section_id(conn: &Connection, section_id: i64) -> rusqlite::Result<Vec<Module>> {
    let mut stmt = conn.prepare("SELECT * FROM Modules WHERE section_id = ?")?;
    let modules_iter = stmt.query_map(params![section_id], |row| {
        Ok(Module {
            id: row.get(0)?,
            name: row.get(1)?,
            instance: row.get(2)?,
            contextid: row.get(3)?,
            description: row.get(4)?,
            contents: get_content_by_module_id(conn, row.get(0)?)?,
            section_id: row.get(6)?,
        })
    })?;

    let mut modules = Vec::new();
    for module_result in modules_iter {
        modules.push(module_result?);
    }

    Ok(modules)
}

fn get_content_by_module_id(conn: &Connection, module_id: i64) -> rusqlite::Result<Option<Vec<Content>>> {
    let mut stmt = conn.prepare("SELECT * FROM Content WHERE module_id = ?")?;
    let content_iter = stmt.query_map(params![module_id], |row| {
        Ok(Content {
            content_type: row.get(0)?,
            filename: row.get(1)?,
            fileurl: row.get(2)?,
            timemodified: row.get(3)?,
            module_id: row.get(4)?,
        })
    })?;

    let mut contents = Vec::new();
    for content_result in content_iter {
        contents.push(content_result?);
    }

    if contents.is_empty() {
        Ok(None)
    } else {
        Ok(Some(contents))
    }
}

pub fn get_sections_by_course_id(conn: &Connection, course_id: i64) -> Result<Vec<Section>> {
    let mut stmt = conn.prepare("SELECT * FROM Sections WHERE courseid = ?")?;
    let sections_iter = stmt.query_map(params![course_id], |row| {
        Ok(Section {
            id: row.get(1)?,
            name: row.get(2)?,
            summary: row.get(3)?,
            courseid: row.get(5)?,
            modules: get_modules_by_section_id(conn, row.get(0)?)?,
        })
    })?;

    let mut sections = Vec::new();
    for section_result in sections_iter {
        sections.push(section_result?);
    }

    Ok(sections)
}
