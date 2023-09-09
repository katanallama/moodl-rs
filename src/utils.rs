use std::path::PathBuf;
use std::fs;
use std::io;

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

