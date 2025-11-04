use crate::ALIAS;
use std::{fmt::Display, path::Path};

pub mod config;
pub mod notification;
pub mod plist;

/// 应用信息结构体
#[derive(Debug)]
pub struct AppInfo {
    /// 应用名称
    pub name: String,
    /// 版本号
    pub version: String,
    /// 短版本
    pub short_version: String,
    /// 应用唯一标识
    pub bundle_id: String,
    /// 检查更新方式
    pub check_update_type: CheckUpType,
}

impl AppInfo {
    /// 是否为 MAS 应用
    pub fn is_mas_app(&self) -> bool {
        matches!(
            self.check_update_type,
            CheckUpType::Mas {
                bundle_id: _,
                is_ios_app: _,
            }
        )
    }
    /// 是否为 Sparkle 分发应用
    pub fn is_sparkle_app(&self) -> bool {
        matches!(self.check_update_type, CheckUpType::Sparkle(_))
    }
}

/// 检查更新方式枚举
#[derive(Debug, PartialEq)]
pub enum CheckUpType {
    /// 已经忽略的应用
    Ignored,
    /// MAS 应用
    Mas { bundle_id: String, is_ios_app: bool },
    /// 使用 Sparkle 查询更新
    Sparkle(String),
    /// 使用 Homebrew 查询更新
    HomeBrew { app_name: String, bundle_id: String },
}

/// 应用信息的格式化输出
impl Display for CheckUpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckUpType::Ignored => write!(f, "已被配置文件忽略的应用"),
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
                let alias_keys = ALIAS.keys();
                let file_name = if !alias_keys.into_iter().any(|x| x == &bundle_id.to_string()) {
                    &dealed_app_name
                } else {
                    &ALIAS[bundle_id]
                };
                write!(f, "检查更新方式为 HomeBrew，获取到的 bundle_id 为: {bundle_id}，默认的信息获取链接为: https://formulae.brew.sh/api/cask/{file_name}.json")
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
    // 排除 `.` 开头的应用，这些应用通常为其他应用的安装产生的，如：.Karabiner-VirtualHIDDevice-Manager.app
    // 排除不是以 .app 结尾的路径
    if !app_name_str.starts_with('.') && app_name_str.ends_with(".app") {
        let content_path = &path.join("Contents");
        let receipt_path = &content_path.join("_MASReceipt");
        let wrapper_path = &path.join("Wrapper");
        let wrapper_plist_path = &path.join("Wrapper/iTunesMetadata.plist");
        let info_plist_path = &content_path.join("Info.plist");
        let name_strs: Vec<&str> = app_name_str.split(".app").collect();
        let name_str = name_strs[0];
        // 如果 `xx.app/Contents/Wrapper` 存在，可以确认这是一个 iOS/iPadOS 应用
        if wrapper_path.exists() {
            // 判断 `xx.app/Contents/Wrapper/iTunesMetadata.plist` 是否存在，如果不存在的话认定为未知应用，跳过
            if wrapper_plist_path.exists() {
                let plist_info = plist::read_plist_info(wrapper_plist_path);
                if config::check_is_ignore(&plist_info.bundle_id) {
                    return Some(AppInfo {
                        name: name_str.to_string(),
                        version: plist_info.version,
                        short_version: plist_info.short_version,
                        bundle_id: plist_info.bundle_id,
                        check_update_type: CheckUpType::Ignored,
                    });
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
                // FIXME: 确认是否应该在这里输出提示
                return None;
            }
        } else {
            let plist_info = plist::read_plist_info(info_plist_path);
            if config::check_is_ignore(&plist_info.bundle_id) {
                return Some(AppInfo {
                    name: name_str.to_string(),
                    version: plist_info.version,
                    short_version: plist_info.short_version,
                    bundle_id: plist_info.bundle_id,
                    check_update_type: CheckUpType::Ignored,
                });
            }
            let cu_type: CheckUpType;
            if receipt_path.exists() {
                // `xx.app/Contents/_MASReceipt` 存在的话为 MAS 下载的应用，通过上面的排除，可以确认为 macOS 独享应用
                // FIXME: OpenCat.app 从官网下载的 2.14.1 版本也会存在 `Contents/_MASReceipt` 目前还没有合适的解决方法
                cu_type = CheckUpType::Mas {
                    bundle_id: plist_info.bundle_id.to_string(),
                    is_ios_app: false,
                };
            } else if let Some(feed_url) = plist_info.feed_url {
                // Info.plist 内存在 SUFeedURL 即为 Sparkle 分发应用
                cu_type = CheckUpType::Sparkle(feed_url);
            } else {
                // 其他应用统一为 Homebrew 查询
                cu_type = CheckUpType::HomeBrew {
                    app_name: name_str.to_string(),
                    // gimp 有一个版本的 bundle_id 是 `org.gimp.gimp-2.10:`
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
