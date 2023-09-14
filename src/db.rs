// db.rs
//
// where to store your database, default is your system data directory
// linux/mac: ~/.local/share/moodl-rs/moodl-rs.db
// windows: %USERPROFILE%/.local/share/moodl-rs/moodl-rs.db
use crate::utils::*;
use eyre::{Result, WrapErr};
use rusqlite::{Connection, Transaction};
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
    fn bind_parameters(&self) -> Vec<(&'static str, &dyn rusqlite::ToSql)>;
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

pub fn create_tables(conn: &rusqlite::Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS Sections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sectionid INTEGER,
            name TEXT,
            summary TEXT,
            lastfetched DATETIME,
            timemodified DATETIME,
            courseid INTEGER,
            UNIQUE(sectionid)
        );",
        (),
    )
    .wrap_err("Failed to create Sections table")?;

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
            timemodified DATETIME,
            UNIQUE(moduleid),
            FOREIGN KEY (section_id) REFERENCES Sections(sectionid)
        );",
        (),
    )
    .wrap_err("Failed to create Modules table")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Content (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            filename TEXT,
            fileurl TEXT,
            localpath TEXT,
            lastfetched DATETIME,
            timemodified DATETIME,
            module_id INTEGER,
            UNIQUE(filename, fileurl, module_id),
            FOREIGN KEY (module_id) REFERENCES Modules(moduleid)
        );",
        (),
    )
    .wrap_err("Failed to create Content table")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Pages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            pageid INTEGER,
            coursemodule INTEGER,
            course INTEGER,
            name TEXT,
            intro TEXT,
            content TEXT,
            revision INTEGER,
            timemodified DATETIME,
            lastfetched DATETIME,
            UNIQUE(pageid)
        );",
        (),
    )
    .wrap_err("Failed to create Pages table")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            filename TEXT,
            fileurl TEXT,
            localpath TEXT,
            timemodified DATETIME,
            lastfetched DATETIME,
            page_id INTEGER,
            UNIQUE(filename, page_id),
            FOREIGN KEY (page_id) REFERENCES Pages(pageid)
        );",
        (),
    )
    .wrap_err("Failed to create Files table")?;

    Ok(())
}
