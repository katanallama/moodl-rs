// utils.rs
//
use eyre::Result;
use fern::InitError;
use std::{
    fs,
    path::{Path, PathBuf},
};
use {
    chrono::Local,
    termimad::{crossterm::style::Color::*, MadSkin, *},
};

#[cfg(not(target_os = "windows"))]
pub fn home_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("$HOME not found");
    PathBuf::from(home)
}

#[cfg(target_os = "windows")]
pub fn home_dir() -> PathBuf {
    let home = std::env::var("USERPROFILE").expect("%userprofile% not found");
    PathBuf::from(home)
}

pub fn config_dir() -> PathBuf {
    let config_dir =
        std::env::var("XDG_CONFIG_HOME").map_or_else(|_| home_dir().join(".config"), PathBuf::from);
    config_dir.join("moodl-rs")
}

pub fn data_dir() -> PathBuf {
    let data_dir = std::env::var("XDG_DATA_HOME")
        .map_or_else(|_| home_dir().join(".local").join("share"), PathBuf::from);

    data_dir.join("moodl-rs")
}

pub fn create_dir(file_path: &str) -> Result<()> {
    let path = Path::new(file_path);
    if let Some(parent_path) = path.parent() {
        if !parent_path.exists() {
            fs::create_dir_all(parent_path)?;
        }
    }
    Ok(())
}

pub fn modify_shortname(shortname: &str) -> String {
    let re = regex::Regex::new(r"(?i)([a-z]+)\s*(\d+)(?:\s*(lab|l))?").unwrap();
    if let Some(caps) = re.captures(shortname) {
        let mut result = format!("{}{}", &caps[1].to_uppercase(), &caps[2]);
        if let Some(suffix) = caps.get(3) {
            result.push_str(&suffix.as_str().to_uppercase());
        }
        return result;
    }
    shortname.to_string()
}

pub fn make_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.table.align = Alignment::Center;
    skin.set_headers_fg(AnsiValue(178));
    skin.bold.set_fg(Yellow);
    skin.italic.set_fg(Magenta);
    skin.scrollbar.thumb.set_fg(AnsiValue(178));
    skin
}

pub fn setup_logger() -> Result<(), InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] [{}] {}",
                Local::now().format("%H:%M:%S"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .level_for("html5ever", log::LevelFilter::Warn)
        .level_for("selectors", log::LevelFilter::Warn)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}
