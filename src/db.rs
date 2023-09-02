// db.rs
//
use crate::models::course_content::{Assignment, CourseSection, Grade, Page};
use crate::models::response::CustomError;
use crate::models::user::User;
use crate::ws::ApiConfig;
use rusqlite::OptionalExtension;

use chrono::Utc;
use rusqlite::{params, Connection, Result};

pub fn initialize_db() -> Result<Connection> {
    let conn = Connection::open("moodl-rs.db")?;
    Ok(conn)
}

pub fn create_user_table(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS User (
            id INTEGER PRIMARY KEY,
            content TEXT NOT NULL,
            privkey TEXT NOT NULL,
            url TEXT NOT NULL,
            wstoken TEXT NOT NULL,
            lastfetched INTEGER
        )",
        (),
    )?;

    println!("[INFO] User table has been created");
    Ok(())
}

pub fn insert_user(
    conn: &mut rusqlite::Connection,
    user: &User,
    api_config: &ApiConfig,
) -> Result<(), CustomError> {
    let tx = conn.transaction()?;

    {
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO User (id, content, privkey, url, wstoken, lastfetched)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;

        stmt.execute(params![
            user.id,
            user.content,
            user.privkey,
            api_config.url,
            api_config.wstoken,
            user.lastfetched,
        ])?;
    }

    tx.commit()?;

    Ok(())
}

pub fn create_course_content_tables(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS Sections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sectionid INTEGER,
            courseid INTEGER,
            modules TEXT,
            name TEXT,
            summary TEXT,
            lastfetched INTEGER,
            UNIQUE(sectionid)
        );",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Modules (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            courseid INTEGER,
            moduleid INTEGER,
            modulename TEXT,
            content TEXT,
            lastfetched INTEGER,
            UNIQUE(moduleid)
        );",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Assignments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            assignid INTEGER,
            courseid INTEGER,
            cmid INTEGER,
            content TEXT NOT NULL,
            lastfetched INTEGER,
            UNIQUE(assignid)
        );",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Grades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            gradeid INTEGER,
            courseid INTEGER,
            cmid INTEGER,
            content TEXT NOT NULL,
            lastfetched INTEGER,
            UNIQUE(gradeid)
        );",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Pages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            courseid INTEGER,
            cmid INTEGER,
            content TEXT NOT NULL,
            lastfetched INTEGER,
            lastmodified INTEGER,
            UNIQUE(cmid)
        );",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS PageHistory (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            cmid INTEGER,
            content TEXT NOT NULL,
            lastmodified INTEGER,
            FOREIGN KEY(cmid) REFERENCES Pages(cmid)
        );",
        (),
    )?;

    Ok(())
}

pub fn insert_content(
    conn: &mut rusqlite::Connection,
    courseid: Option<i32>,
    sections: &[CourseSection],
) -> Result<(), CustomError> {
    let tx = conn.transaction()?;

    {
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO Sections (sectionid, courseid, name, summary, lastfetched)
                VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;

        for section in sections {
            stmt.execute(params![
                section.sectionid,
                courseid,
                section.name,
                section.summary,
                section.lastfetched
            ])?;

            let mut module_stmt = tx.prepare(
                "INSERT OR REPLACE INTO Modules (moduleid, courseid, modulename, content, lastfetched)
                    VALUES (?1, ?2, ?3, ?4, ?5)
                    ON CONFLICT(moduleid) DO UPDATE SET
                        lastfetched=excluded.lastfetched,
                        modulename=excluded.modulename,
                        content=excluded.content"
            )?;

            for module in &section.modules {
                module_stmt.execute(params![
                    module.moduleid,
                    courseid,
                    module.modulename,
                    module.content,
                    module.lastfetched
                ])?;
            }
        }
    }

    tx.commit()?;

    Ok(())
}

pub fn insert_pages(conn: &mut rusqlite::Connection, pages: &[Page]) -> Result<(), CustomError> {
    let tx = conn.transaction()?;

    {
        let mut check_content_stmt = tx.prepare("SELECT content FROM Pages WHERE cmid = ?1")?;

        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO Pages (courseid, cmid, content, lastfetched, lastmodified)
                VALUES (?1, ?2, ?3, ?4, ?5)
                ON CONFLICT(cmid) DO UPDATE SET
                    content=excluded.content,
                    lastfetched=excluded.lastfetched,
                    lastmodified=excluded.lastmodified",
        )?;

        // audit trail of changes for the content of each page
        let mut history_stmt = tx.prepare(
            "INSERT INTO PageHistory (cmid, content, lastmodified)
                VALUES (?1, ?2, ?3)",
        )?;

        let mut update_lastfetched_stmt =
            tx.prepare("UPDATE Pages SET lastfetched = ?1 WHERE cmid = ?2")?;

        let now = Utc::now().timestamp();

        for page in pages {
            let current_content: Option<String> = check_content_stmt
                .query_row(params![page.cmid], |row| row.get(0))
                .optional()?;

            if current_content != Some(page.content.clone()) {
                // Content has changed, so insert/replace and log to history
                stmt.execute(params![
                    page.courseid,
                    page.cmid,
                    page.content,
                    page.lastfetched,
                    now
                ])?;

                history_stmt.execute(params![page.cmid, page.content, now])?;
            } else {
                // Content hasn't changed, so just update lastfetched
                update_lastfetched_stmt.execute(params![page.lastfetched, page.cmid,])?;
            }
        }
    }

    tx.commit()?;

    Ok(())
}

pub fn insert_assignments(
    conn: &mut rusqlite::Connection,
    assignments: &[Assignment],
) -> Result<(), CustomError> {
    let tx = conn.transaction()?;

    {
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO Assignments (assignid, courseid, cmid, content, lastfetched)
                VALUES (?1, ?2, ?3, ?4, ?5)
                ON CONFLICT(assignid) DO UPDATE SET
                    lastfetched=excluded.lastfetched,
                    content=excluded.content",
        )?;

        for assign in assignments {
            stmt.execute(params![
                assign.assignid,
                assign.courseid,
                assign.cmid,
                assign.content,
                assign.lastfetched
            ])?;
        }
    }

    tx.commit()?;

    Ok(())
}

pub fn insert_grades(conn: &mut rusqlite::Connection, grades: &[Grade]) -> Result<(), CustomError> {
    let tx = conn.transaction()?;

    {
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO Grades (gradeid, courseid, cmid, content, lastfetched)
                VALUES (?1, ?2, ?3, ?4, ?5)
                ON CONFLICT(gradeid) DO UPDATE SET
                    lastfetched=excluded.lastfetched,
                    content=excluded.content",
        )?;
        for grd in grades {
            stmt.execute(params![
                grd.gradeid,
                grd.courseid,
                grd.cmid,
                grd.content,
                grd.lastfetched
            ])?;
        }
    }

    tx.commit()?;

    Ok(())
}

pub fn get_user(conn: &Connection, id: Option<i32>) -> Result<Option<(i32, String, String)>> {
    let sql;
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(user_id) = id {
        sql = "SELECT id, wstoken, url FROM User WHERE id = ?1";
        params.push(Box::new(user_id));
    } else {
        sql = "SELECT id, wstoken, url FROM User LIMIT 1";
    }

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(AsRef::as_ref).collect();

    let mut stmt = conn.prepare(sql)?;
    let mut user_iter = stmt.query_map(&*params_refs, |row| {
        Ok((
            row.get(0)?, // userid
            row.get(1)?, // wstoken
            row.get(2)?, // url
        ))
    })?;

    if let Some(user) = user_iter.next() {
        Ok(Some(user?))
    } else {
        Ok(None)
    }
}
