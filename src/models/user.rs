// models/user.rs
//
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Function {
    name: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdvancedFeature {
    name: String,
    value: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SiteInfo {
    pub sitename: String,
    pub username: String,
    pub firstname: String,
    pub lastname: String,
    pub fullname: String,
    pub userid: i64,
    pub siteurl: String,
    pub userpictureurl: String,
    pub functions: Vec<Function>,
    pub release: String,
    pub version: String,
    pub mobilecssurl: String,
    pub advancedfeatures: Vec<AdvancedFeature>,
}
