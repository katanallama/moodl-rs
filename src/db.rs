// db.rs
//
// where to store your database, default is your system data directory
// linux/mac: ~/.local/share/moodl-rs/moodl-rs.db
// windows: %USERPROFILE%/.local/share/moodl-rs/moodl-rs.db
use crate::utils::*;
use eyre::{Result, WrapErr};
use rusqlite::types::ToSql;
use rusqlite::{params, Connection, Transaction};
use std::fs;

pub fn initialize_db() -> Result<()> {
    let data_directory = data_dir();

    if !data_directory.exists() {
        fs::create_dir_all(&data_directory).wrap_err("Failed to create data directory")?;
    }

    let db_path = data_directory.join("moodl-rs.db");

    let conn = Connection::open(db_path).wrap_err("Failed to open connection to the database")?;
    create_tables(&conn).wrap_err("Failed to create tables in the database")?;

    Ok(())
}

pub fn connect_db() -> Result<Connection> {
    let db_path = data_dir().join("moodl-rs.db");
    let conn = Connection::open(db_path).wrap_err("Failed to connect to the database")?;
    Ok(conn)
}

pub trait Insertable {
    fn insert_query() -> &'static str;
    fn bind_parameters(&self) -> Vec<(&'static str, &dyn ToSql)>;
}

pub fn generic_insert<T: Insertable>(tx: &Transaction, item: &T) -> Result<()> {
    let mut stmt = tx
        .prepare(T::insert_query())
        .wrap_err_with(|| format!("Failed to prepare query: {}", T::insert_query()))?;

    let params = item.bind_parameters();
    stmt.execute(&params[..]).wrap_err_with(|| {
        let param_keys = params
            .iter()
            .map(|(k, _)| k.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        format!("Failed to execute query with parameters: {}", param_keys)
    })?;

    Ok(())
}

pub trait Retrievable {
    fn select_query() -> &'static str;
    fn select_query_all() -> &'static str;
    fn from_row(row: &rusqlite::Row) -> Result<Self>
    where
        Self: Sized;
}

pub fn generic_retrieve<T: Retrievable>(tx: &Transaction) -> Result<Vec<T>> {
    let mut stmt = tx
        .prepare(T::select_query_all())
        .wrap_err_with(|| format!("Failed to prepare query: {}", T::select_query_all()))?;

    let mut rows = stmt.query(params![])?;
    let mut results = Vec::new();

    while let Some(row) = rows.next()? {
        results.push(T::from_row(row)?);
    }

    Ok(results)
}

pub fn retrieve_param<T: Retrievable>(tx: &Transaction, params: &[&(dyn ToSql)]) -> Result<Vec<T>> {
    let mut stmt = tx
        .prepare(T::select_query())
        .wrap_err_with(|| format!("Failed to prepare query: {}", T::select_query()))?;

    let mut rows = stmt.query(params)?;
    let mut results = Vec::new();

    while let Some(row) = rows.next()? {
        results.push(T::from_row(row)?);
    }

    Ok(results)
}

pub fn create_tables(conn: &rusqlite::Connection) -> Result<()> {
        conn.execute(
        "CREATE TABLE IF NOT EXISTS Assignments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            assignid INTEGER,
            cmid INTEGER,
            course INTEGER,
            name TEXT,
            duedate DATETIME,
            submissionsopen DATETIME,
            timemodified DATETIME,
            cutoffdate DATETIME,
            intro TEXT,
            lastfetched DATETIME,
            courseid INTEGER,
            UNIQUE(assignid)
        );",
        (),
    )
    .wrap_err("Failed to create Sections table")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Grades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            gradeid INTEGER,
            itemname TEXT,
            itemmodule TEXT,
            iteminstance INTEGER,
            itemnumber INTEGER,
            idnumber TEXT,
            categoryid INTEGER,
            cmid INTEGER,
            graderaw INTEGER,
            gradedatesubmitted DATETIME,
            gradedategraded DATETIME,
            grademin INTEGER,
            grademax INTEGER,
            feedback TEXT,
            lastfetched DATETIME,
            courseid INTEGER,
            UNIQUE(gradeid)
        );",
        (),
    )
    .wrap_err("Failed to create Grades table")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Sections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sectionid INTEGER,
            name TEXT,
            summary TEXT,
            lastfetched DATETIME,
            courseid INTEGER,
            UNIQUE(sectionid)
        );",
        (),
    )
    .wrap_err("Failed to create Sections table")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Scorms (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            scormid INTEGER,
            coursemodule INTEGER,
            name TEXT,
            intro TEXT,
            packageurl TEXT,
            localpath TEXT,
            version INTEGER,
            maxgrade INTEGER,
            grademethod INTEGER,
            whatgrade INTEGER,
            maxattempt INTEGER,
            lastfetched DATETIME,
            courseid INTEGER,
            UNIQUE(scormid)
        );",
        (),
    )
    .wrap_err("Failed to create Scorms table")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Modules (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            moduleid INTEGER,
            name TEXT,
            instance INTEGER,
            contextid INTEGER,
            description TEXT,
            lastfetched DATETIME,
            section_id INTEGER,
            UNIQUE(moduleid),
            FOREIGN KEY (section_id) REFERENCES Sections(sectionid)
        );",
        (),
    )
    .wrap_err("Failed to create Modules table")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            filename TEXT,
            fileurl TEXT,
            localpath TEXT,
            timemodified DATETIME,
            lastfetched DATETIME,
            module_id INTEGER,
            UNIQUE(filename),
            FOREIGN KEY (module_id) REFERENCES Modules(moduleid)
        );",
        (),
    )
    .wrap_err("Failed to create Files table")?;

    Ok(())
}
