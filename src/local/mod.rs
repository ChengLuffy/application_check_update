use std::{fmt::Display, path::Path};

pub mod config;
pub mod notification;
pub mod plist;

#[derive(Debug)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub short_version: String,
    pub bundle_id: String,
    pub check_update_type: CheckUpType,
}

#[derive(Debug, PartialEq)]
pub enum CheckUpType {
    Mas { bundle_id: String, is_ios_app: bool },
    Sparkle(String),
    HomeBrew { app_name: String, bundle_id: String },
}

impl Display for CheckUpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckUpType::Mas {
                bundle_id,
                is_ios_app,
            } => {
                let temp = if *is_ios_app {
                    "，这是一个 iOS/iPadOS/Mac-Catalyst 应用"
                } else {
                    ""
                };
                write!(
                    f,
                    "检查更新方式为 iTunes API，获取到的 bundle_id 为: {bundle_id}{temp}"
                )
            }
            CheckUpType::Sparkle(feed_url) => write!(
                f,
                "检查更新方式为 Sparkle，获取到的 SUFeedURL 为: {feed_url}"
            ),
            CheckUpType::HomeBrew {
                app_name,
                bundle_id,
            } => {
                let dealed_app_name = app_name.to_lowercase().replace(' ', "-");
                write!(f, "检查更新方式为 HomeBrew，获取到的 bundle_id 为: {bundle_id}，默认的信息获取链接为: https://formulae.brew.sh/api/cask/{dealed_app_name}.json")
            }
        }
    }
}

/// 查询应用类型
///
/// - 未能识别的应用类型将跳过查询
/// - 包内存在 `_MASReceipt` 路径判断为 MAS 应用
/// - 包内存在 `Wrapper/iTunesMetadata.plist` 路径判断为 iOS 应用
/// - `Info.plist` 中存在 `SUFeedURL` 字段判断为依赖 `Sparkle` 检查更新的应用
/// - 其他应用通过 `HomeBrew-Casks` 查询版本号
pub fn check_app_info(entry: &Path) -> Option<AppInfo> {
    let path = entry;
    let app_name = path.file_name().unwrap_or_default();
    let app_name_str = app_name.to_str().unwrap_or_default();
    if !app_name_str.starts_with('.') && app_name_str.ends_with(".app") {
        let content_path = &path.join("Contents");
        let receipt_path = &content_path.join("_MASReceipt");
        let wrapper_path = &path.join("Wrapper");
        let wrapper_plist_path = &path.join("Wrapper/iTunesMetadata.plist");
        let info_plist_path = &content_path.join("Info.plist");
        let name_strs: Vec<&str> = app_name_str.split(".app").collect();
        let name_str = name_strs[0];
        if wrapper_path.exists() {
            if wrapper_plist_path.exists() {
                let plist_info = plist::read_plist_info(wrapper_plist_path);
                if config::check_is_ignore(&plist_info.bundle_id) {
                    return None;
                }
                let cu_type = CheckUpType::Mas {
                    bundle_id: plist_info.bundle_id.to_string(),
                    is_ios_app: true,
                };
                let app_info = AppInfo {
                    name: name_str.to_string(),
                    version: plist_info.version,
                    short_version: plist_info.short_version,
                    bundle_id: plist_info.bundle_id,
                    check_update_type: cu_type,
                };
                return Some(app_info);
            } else {
                return None;
            }
        } else {
            let plist_info = plist::read_plist_info(info_plist_path);
            if config::check_is_ignore(&plist_info.bundle_id) {
                return None;
            }
            let cu_type: CheckUpType;
            if receipt_path.exists() {
                cu_type = CheckUpType::Mas {
                    bundle_id: plist_info.bundle_id.to_string(),
                    is_ios_app: false,
                };
            } else if let Some(feed_url) = plist_info.feed_url {
                cu_type = CheckUpType::Sparkle(feed_url);
            } else {
                cu_type = CheckUpType::HomeBrew {
                    app_name: name_str.to_string(),
                    bundle_id: plist_info.bundle_id.replace(':', ""),
                };
            }
            let app_info = AppInfo {
                name: name_str.to_string(),
                version: plist_info.version.to_string(),
                short_version: plist_info.short_version.to_string(),
                bundle_id: plist_info.bundle_id.to_string(),
                check_update_type: cu_type,
            };
            return Some(app_info);
        }
    }
    None
}
