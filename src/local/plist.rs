use plist::Value;
use std::path::PathBuf;

/// plist 内应用信息结构体
pub struct InfoPlistInfo {
    pub version: String,
    pub short_version: String,
    pub bundle_id: String,
    pub feed_url: Option<String>,
}

/// 从应用包内 `Info.plist` 文件中读取有用信息
pub fn read_plist_info(plist_path: &PathBuf) -> InfoPlistInfo {
    let mut short_version_key_str = "CFBundleShortVersionString";
    let mut version_key_str = "CFBundleVersion";
    let mut bundle_id_key_str = "CFBundleIdentifier";
    let feed_url_key = "SUFeedURL";
    // 如果不是是 Info.plist 文件，另外一套 key
    if !plist_path.ends_with("Info.plist") {
        short_version_key_str = "bundleShortVersionString";
        version_key_str = "bundleVersion";
        bundle_id_key_str = "softwareVersionBundleId";
    }
    let value = Value::from_file(plist_path)
        .unwrap_or_else(|_| panic!("plist 文件读取错误 {plist_path:?}"));
    // 读取 bundle_id
    let bundle_id = value
        .as_dictionary()
        .and_then(|dict| dict.get(bundle_id_key_str))
        .and_then(|id| id.as_string())
        .unwrap_or("");
    // 读取 version
    let version = value
        .as_dictionary()
        .and_then(|dict| dict.get(version_key_str))
        .and_then(|id| id.as_string())
        .unwrap_or("");
    // 读取 short_version
    let short_version = value
        .as_dictionary()
        .and_then(|dict| dict.get(short_version_key_str))
        .and_then(|id| id.as_string())
        .unwrap_or("");
    // 如果 bundle_id 和版本信息为空，则换一个路径重新读取
    if bundle_id.is_empty() || (short_version.is_empty() && version.is_empty()) {
        let info_plist_path = plist_path
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("WrappedBundle/Info.plist");
        return read_plist_info(&info_plist_path);
    }
    // 读取 SUFeedURL
    let feed_url_option = value
        .as_dictionary()
        .and_then(|dict| dict.get(feed_url_key))
        .and_then(|id| id.as_string());
    let feed_url = feed_url_option.map(|string| string.to_string());
    InfoPlistInfo {
        version: version.to_string(),
        short_version: short_version.to_string(),
        bundle_id: bundle_id.to_string(),
        feed_url,
    }
}

/// 获取系统版本
pub fn get_arm_system_version() -> String {
    let info = Value::from_file("/System/Library/CoreServices/SystemVersion.plist")
        .expect("/System/Library/CoreServices/SystemVersion.plist 不存在");
    let product_version = info
        .as_dictionary()
        .and_then(|dict| dict.get("ProductVersion"))
        .and_then(|id| id.as_string())
        .unwrap_or("");
    if product_version.starts_with("14") {
        return "arm64_sonoma".to_string();
    } else if product_version.starts_with("13") {
        return "arm64_ventura".to_string();
    } else if product_version.starts_with("12") {
        return "arm64_monterey".to_string();
    } else if product_version.starts_with("11") {
        return "arm64_big_sur".to_string();
    }
    "".to_string()
}
