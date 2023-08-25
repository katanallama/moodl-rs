// db.rs
//
use crate::models::course::Course;
use crate::models::course_grades::Table;
use rusqlite::{Connection, Result};

pub fn initialize_db() -> Result<Connection> {
    let conn = Connection::open("moodl-rs.db")?;

    Ok(conn)
}

pub fn create_user_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS user (
             id INTEGER PRIMARY KEY,
             wstoken TEXT NOT NULL,
             url TEXT NOT NULL
        )",
        (),
    )?;

    println!("[INFO] User table has been created");
    Ok(())
}

pub fn create_courses_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS courses (
             id INTEGER PRIMARY KEY,
             fullname TEXT NOT NULL,
             summary TEXT NOT NULL,
             lastaccess INTEGER,
             timemodified INTEGER NOT NULL,
             lastfetched INTEGER,
             UNIQUE(id)
         );",
        (),
    )?;

    println!("[INFO] Course table has been created");
    Ok(())
}

pub fn create_grades_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS grades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            courseid INTEGER NOT NULL,
            itemname TEXT NOT NULL,
            grade TEXT NOT NULL,
            feedback TEXT,
            lastfetched INTEGER,
            UNIQUE(itemname)
        );",
        (),
    )?;

    Ok(())
}

pub fn insert_user(conn: &Connection, id: i32, wstoken: String, url: String) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO user (id, wstoken, url) VALUES (?1, ?2, ?3)",
        (id, &wstoken, &url),
    )?;

    println!(
        "[INFO] User has been created:\n  ID:\t {} \n  Key:\t {} \n  URL:\t {}",
        id, &wstoken, &url
    );

    Ok(())
}

pub fn insert_course(conn: &Connection, course: &Course) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO courses (
            id,
            fullname,
            summary,
            lastaccess,
            timemodified
        )
        VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            course.id,
            &course.fullname,
            &course.summary,
            course.lastaccess,
            course.timemodified,
        ),
    )?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS course_lastfetched_after_update
            AFTER INSERT ON courses
            FOR EACH ROW
            BEGIN
                UPDATE courses SET lastfetched = strftime('%s', 'now') WHERE id = NEW.id;
            END;",
        [],
    )?;

    println!(
        "[INFO] {} has been created: {}",
        course.shortname, &course.id
    );

    Ok(())
}

pub fn insert_grade(conn: &Connection, table: &Table) -> Result<()> {
    for data in &table.tabledata {
        if data.itemname.is_some() && data.grade.is_some()
        // && !data.itemname.as_ref().and_then(|d| d.content.as_ref()).contains("Aggregation")
        {
            conn.execute(
                "INSERT OR REPLACE INTO grades (
                    courseid,
                    itemname,
                    grade,
                    feedback
                )
                VALUES (?1, ?2, ?3, ?4)",
                (
                    table.courseid,
                    data.itemname.as_ref().and_then(|d| d.content.as_ref()),
                    data.grade.as_ref().and_then(|d| d.content.as_ref()),
                    data.feedback.as_ref().and_then(|d| d.content.as_ref()),
                ),
            )?;
        }
    }

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS grades_lastfetched_after_update
            AFTER INSERT ON grades
            FOR EACH ROW
            BEGIN
                UPDATE grades SET lastfetched = strftime('%s', 'now') WHERE id = NEW.id;
            END;",
        [],
    )?;

    println!(
        "[INFO] Grades for user {} in course {} have been inserted.",
        table.userid, table.courseid
    );

    Ok(())
}

pub fn _get_all_courses(conn: &Connection) -> Result<Vec<Course>> {
    let mut courses = Vec::new();

    let mut stmt = conn.prepare("SELECT * FROM courses")?;

    let course_rows = stmt.query_map([], |row| {
        Ok(Course {
            id: row.get(0)?,
            shortname: row.get(1)?,
            fullname: row.get(2)?,
            displayname: row.get(3)?,
            idnumber: row.get(4)?,
            summary: row.get(5)?,
            startdate: row.get(6)?,
            enddate: row.get(7)?,
            lastaccess: row.get(8)?,
            showactivitydates: row.get(9)?,
            timemodified: row.get(10)?,
            format: None,
            category: None,
            completed: None,
            completionhascriteria: None,
            completionusertracked: None,
            showcompletionconditions: None,
            showgrades: None,
            marker: None,
            hidden: None,
            isfavourite: None,
            enablecompletion: None,
            lang: None,
            progress: None,
            summaryformat: None,
            visible: None,
        })
    })?;

    for course_row in course_rows {
        courses.push(course_row?);
    }

    Ok(courses)
}

pub fn get_grades(
    conn: &Connection,
    courseid: Option<i32>,
) -> Result<Vec<(Option<String>, Option<String>, Option<String>)>> {
    let mut grades = Vec::new();

    let mut stmt =
        conn.prepare("SELECT itemname, grade, feedback FROM grades WHERE courseid = ?1")?;
    let grade_rows = stmt.query_map([courseid], |row| {
        Ok((
            row.get::<_, Option<String>>(0)?, // itemname
            row.get::<_, Option<String>>(1)?, // grade
            row.get::<_, Option<String>>(2)?, // feedback
        ))
    })?;

    for grade_row in grade_rows {
        grades.push(grade_row?);
    }

    Ok(grades)
}

pub fn get_user(conn: &Connection, id: Option<i32>) -> Result<Option<(i32, String, String)>> {
    let sql;
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(user_id) = id {
        sql = "SELECT id, wstoken, url FROM user WHERE id = ?1";
        params.push(Box::new(user_id));
    } else {
        sql = "SELECT id, wstoken, url FROM user LIMIT 1";
    }

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(AsRef::as_ref).collect();

    let mut stmt = conn.prepare(sql)?;
    let mut user_iter = stmt.query_map(&*params_refs, |row| {
        Ok((
            row.get(0)?, // id
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
