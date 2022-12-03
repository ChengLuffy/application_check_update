use core::str;
use std::{fs::{self, DirEntry}, path::{Path, PathBuf}, cmp};
use plist::Value;
use lazy_static::lazy_static;
use threadpool::ThreadPool;
use rss::Channel;
use yaml_rust::yaml;

/// TODO: 尝试使用 tui 输出
/// TODO: 支持单应用查询
/// TODO: 支持并行查询数量

lazy_static! {
    static ref IGNORES: yaml::Yaml = get_ignore_config();
    static ref ALIAS: yaml::Yaml = get_alias_config();
    static ref SYSTEM_NAME: String = get_system_version();
}

fn main() {
    let apps_path = Path::new("/Applications");
    let n_workers = 5;
    let pool = ThreadPool::new(n_workers);
    for item in fs::read_dir(apps_path).unwrap() {
        // 直接使用 thread::spawn 会产生 `Too many open files` 的问题，也不知道这是不是合适的解决方法
        pool.execute(move|| {
            match item {
                Ok(path) => {
                    // if path.path().file_name().unwrap_or_default().to_str() != Some("饿了么.app") {
                    //     println!("{:?}", path);
                    //     continue;
                    // }
                    let app_info = check_app_info(&path);
                    match app_info {
                        Some(info) => check_update(info),
                        None => () // println!("{:?} 无法解析应用信息", &path)
                    }
                },
                Err(error) => println!("{:?}", error)
            }
        });
    }
    pool.join();

    // let remote_info = sparkle_app_check("https://api.appcenter.ms/v0.1/public/sparkle/apps/1cd052f7-e118-4d13-87fb-35176f9702c1");
    // println!("{}\n{}", remote_info.update_page_url, remote_info.version);
    // let remote_info = homebrew_check("parallels desktop", "com.parallels.desktop.console");
    // println!("{}\n{}", remote_info.update_page_url, remote_info.version);
    // let remote_info = sparkle_feed("https://raw.githubusercontent.com/xjbeta/AppUpdaterAppcasts/master/Aria2D/Appcast.xml");
    // println!("{}\n{}", remote_info.update_page_url, remote_info.version);
}

/// 根据应用类型查询更新并输出
fn check_update(app_info: AppInfo) {
    let check_update_type = &app_info.check_update_type;
    let mut remote_info: RemoteInfo;
    loop {
        remote_info = match check_update_type {
            CheckUpType::MAS(bundle_id) =>  area_check(bundle_id), 
            CheckUpType::Sparkle(feed_url) => sparkle_feed(feed_url),
            CheckUpType::HomeBrew {app_name, bundle_id} => homebrew_check(app_name, bundle_id)
            // _ => RemoteInfo { version: "-2".to_string(), update_page_url: String::new() }
        };
        if &remote_info.version == "-1" {
            continue;
            // break;
        } else {
            break;
        }
    }
    if remote_info.version.len() == 0 {
        println!("=====");
        println!("{}", app_info.name);
        println!("{:?}", app_info.check_update_type);
        println!("local version {}", app_info.version);
        println!("remote version check failed");
        println!("=====\n");
    }
    let ordering = cmp_version(&app_info.version, &remote_info.version, false);
    if ordering.is_lt() {
    // if &remote_info.version != "-2" {
        println!("=====");
        println!("{}", app_info.name);
        println!("local version {}", app_info.version);
        println!("remote version {}", remote_info.version);
        println!("{}", remote_info.update_page_url);
        println!("=====\n");
    }
}

/// 查询应用类型
/// 
/// - 未能识别的应用类型将跳过查询
/// - 包内存在 `_MASReceipt` 路径判断为 MAS 应用
/// - 包内存在 `Wrapper/iTunesMetadata.plist` 路径判断为 iOS 应用
/// - `Info.plist` 中存在 `SUFeedURL` 字段判断为依赖 `Sparkle` 检查更新的应用
/// - 其他应用通过 `HomeBrew-Casks` 查询版本号
fn check_app_info(entry: &DirEntry) -> Option<AppInfo> {
    let path = entry.path();
    let app_name = path.file_name().unwrap_or_default();
    let app_name_str = app_name.to_str().unwrap_or_default();
    if !app_name_str.starts_with(".") && app_name_str.ends_with(".app") {
        let content_path = &path.join("Contents");
        let receipt_path = &content_path.join("_MASReceipt");
        let wrapper_path = &path.join("Wrapper/iTunesMetadata.plist");
        let info_plist_path = &content_path.join("Info.plist");
        let name_strs: Vec<&str> = app_name_str.split(".app").collect();
        let name_str = name_strs[0];
        if wrapper_path.exists() {
            let plist_info = read_plist_info(wrapper_path);
            if check_is_ignore(&plist_info.bundle_id) {
                return None;
            }
            let cu_type = CheckUpType::MAS(plist_info.bundle_id.to_string());
            let app_info = AppInfo {
                name: name_str.to_string(),
                version: plist_info.version.to_string(),
                check_update_type: cu_type
            };
            return Some(app_info);
        } else {
            let plist_info = read_plist_info(info_plist_path);
            if check_is_ignore(&plist_info.bundle_id) {
                return None;
            }
            let cu_type: CheckUpType;
            if receipt_path.exists() {
                cu_type = CheckUpType::MAS(plist_info.bundle_id.to_string());
            } else if let Some(feed_url) = plist_info.feed_url {
                cu_type = CheckUpType::Sparkle(feed_url.to_string());
            } else {
                cu_type = CheckUpType::HomeBrew {
                    app_name: name_str.to_string(),
                    bundle_id: plist_info.bundle_id.replace(":", "").to_string()
                };
            }
            let app_info = AppInfo {
                name: name_str.to_string(), 
                version: plist_info.version.to_string(), 
                check_update_type: cu_type.into()
            };
            return Some(app_info);
        }
    }
    return None;
}

/// TODO: 通过配置文件配置备选区域代码
/// MAS 应用和 iOS 应用可能存在区域内未上架的问题，采取先检测 cn 后检测 us 的方式
fn area_check(bundle_id: &str) -> RemoteInfo {
    let remote_info_opt = mas_app_check("cn", bundle_id);
    if let Some(remote_info) = remote_info_opt {
        return remote_info;
    }
    let remote_info_opt1 = mas_app_check("us", bundle_id);
    if let Some(remote_info) = remote_info_opt1 {
        return remote_info;
    }
    return RemoteInfo {version: String::new(), update_page_url: "".to_string()};
}

/// 从 `Info.plist` 文件中读取有用信息
fn read_plist_info(plist_path: &PathBuf) -> InfoPlistInfo {
    let mut short_version_key_str = "CFBundleShortVersionString";
    let mut version_key_str = "CFBundleVersion";
    let mut bundle_id_key_str = "CFBundleIdentifier";
    let feed_url_key = "SUFeedURL";
    if plist_path.ends_with("Info.plist") == false {
        short_version_key_str = "bundleShortVersionString";
        version_key_str = "bundleShortVersionString";
        bundle_id_key_str = "softwareVersionBundleId";
    }
    let value = Value::from_file(plist_path).expect("failed to read plist file");
    let bundle_id = value
                            .as_dictionary()
                            .and_then(|dict| dict.get(bundle_id_key_str))
                            .and_then(|id| id.as_string()).unwrap_or("");
    if bundle_id.len() == 0 {
        let info_plist_path = plist_path.parent().unwrap().parent().unwrap().join("WrappedBundle/Info.plist");
        return read_plist_info(&info_plist_path);
    }
    let mut version = value
                                .as_dictionary()
                                .and_then(|dict| dict.get(short_version_key_str))
                                .and_then(|id| id.as_string()).unwrap_or("");
    if version.len() == 0 {
        version = value
                    .as_dictionary()
                    .and_then(|dict| dict.get(version_key_str))
                    .and_then(|id| id.as_string()).unwrap_or("");
    }
    let feed_url_option = value
                    .as_dictionary()
                    .and_then(|dict| dict.get(feed_url_key))
                    .and_then(|id| id.as_string());
    let feed_url = match feed_url_option {
        Some(string) => Some(string.to_string()),
        None => None
    };
    InfoPlistInfo {version: version.to_string(), bundle_id: bundle_id.to_string(), feed_url}
}

////////////////////////////////////////////////////////////////////////////////
// 获取配置信息
////////////////////////////////////////////////////////////////////////////////

/// 获取配置信息
/// 
/// - 配置文件，使用 `bundle id` 确定相应的应用，两种使用场景
/// - 1. 忽略应用，比如企业证书分发的应用，还有无法通过应用商店、Sparkle方式、HomeBrew-Casks 查询到应用版本信息的应用，或者不想检查更新的应用；
/// - 2. HomeBrew-Casks 检测时的别名，大部分应用需要配置
fn get_config() -> yaml::Yaml {
    let mut path = dirs::home_dir().expect("未能定位到用户目录");
    path.push(".config/appcu/config.yaml");
    let content = fs::read_to_string(path).expect("读取配置文件时发生错误");
    let configs = yaml_rust::YamlLoader::load_from_str(&content).expect("解析配置文件时发生错误");
    let config = configs.get(0).expect("解析配置文件时发生错误");
    return config.to_owned();
}

/// 获取别名配置
fn get_alias_config() -> yaml::Yaml {
    let conf = get_config();
    let section = &conf["alias"];
    section.to_owned()
}

/// 获取忽略配置
fn get_ignore_config() -> yaml::Yaml {
    let conf = get_config();
    let section = &conf["ignore"];
    section.to_owned()
}

/// 查询是否是忽略应用
fn check_is_ignore(bundle_id: &str) -> bool {
    let arr = IGNORES.as_vec().unwrap();
    let ignores: Vec<&str> = arr.iter().map(|item| {
        item.as_str().unwrap_or("").trim()
    }).collect();
    let ret = ignores.contains(&bundle_id);
    return ret
}
/// 获取系统版本
fn get_system_version() -> String {
    let info = Value::from_file("/System/Library/CoreServices/SystemVersion.plist").expect("/System/Library/CoreServices/SystemVersion.plist 不存在");
    let product_version = info
                                .as_dictionary()
                                .and_then(|dict| dict.get("ProductVersion"))
                                .and_then(|id| id.as_string()).unwrap_or("");
    if product_version.starts_with("13") {
        return "arm64_ventura".to_string();
    } else if product_version.starts_with("12") {
        return "arm64_monterey".to_string();
    } else if product_version.starts_with("11") {
        return "arm64_big_sur".to_string();
    }
    return "".to_string();
}

////////////////////////////////////////////////////////////////////////////////
// 网络请求
////////////////////////////////////////////////////////////////////////////////

/// 查询 HomeBrew-Casks 内的版本信息
/// 
/// 读取 `Homebrew/homebrew-cask` 仓库 `Casks` 文件夹内的响应应用文件
#[tokio::main]
async fn homebrew_check(app_name: &str, bundle_id: &str) -> RemoteInfo {
    let dealed_app_name = app_name.to_lowercase().replace(" ", "-");
    // println!("{}: {:?}", bundle_id, PROPERTIES.get(bundle_id));
    let file_name = match ALIAS[bundle_id].as_str() {
        Some(alias_name) => alias_name,
        None => &dealed_app_name
    };
    if let Ok(resp) = reqwest::get(format!("https://formulae.brew.sh/api/cask/{}.json", file_name)).await {
        if let Ok(text) = resp.text().await {
            let json_value: serde_json::Value = serde_json::from_str(&text).unwrap();
            let version_arr: Vec<&str> = json_value.get("version").unwrap().as_str().unwrap().split(",").collect();
            let version: &str = version_arr.get(0).unwrap_or(&"");
            let arch_str = std::env::consts::ARCH;
            let mut url = json_value.get("url").unwrap().as_str().unwrap_or_default().to_string();
            if SYSTEM_NAME.len() > 0 {
                if arch_str == "aarch64" || arch_str == "arm" {
                    if let Some(variations) = json_value.get("variations") {
                        if let Some(arm64_ventura) = variations.get(SYSTEM_NAME.as_str()) {
                            if let Some(url_value) = arm64_ventura.get("url") {
                                let url_temp = url_value.as_str().unwrap_or_default().to_string();
                                url = url_temp;
                            }
                        }
                    }
                }
            }
            return RemoteInfo {
                version: version.to_string(),
                update_page_url: url.to_string()
            };
        }
    }
    RemoteInfo {
        version: "-1".to_string(),
        update_page_url: String::new()
    }
}

#[tokio::main]
async fn sparkle_feed(feed_url: &str) -> RemoteInfo {
    if let Ok(content) = reqwest::get(feed_url).await {
        if let Ok(bytes_content) = content.bytes().await {
            if let Ok(channel) = Channel::read_from(&bytes_content[..]) {
                let mut items: Vec<rss::Item> = channel.items().into();
                items.sort_by(|a, b| {
                    if let Some(a_enclosure) = a.enclosure() {
                        if let Some(b_enclosure) = b.enclosure() {
                            let mut a_version = a_enclosure.version.as_str();
                            if !a_version.contains(".") {
                                a_version = &a_enclosure.short_version.as_str();
                            }
                            if a_version.len() == 0 {
                                a_version = a.title().unwrap_or_default();
                            }
                            let mut b_version = b_enclosure.version.as_str();
                            if !b_version.contains(".") {
                                b_version = &b_enclosure.short_version.as_str();
                            }
                            if b_version.len() == 0 {
                                b_version = b.title().unwrap_or_default();
                            }
                            return cmp_version(&a_version, &b_version, true);
                        }
                    }
                    return std::cmp::Ordering::Equal;
                });
                // println!("{:?}", &items);
                if let Some(item) = items.last() {
                    if let Some(enclosure) = item.enclosure() {
                        let mut version = enclosure.version.as_str();
                        if !version.contains(".") {
                            version = &enclosure.short_version;
                        }
                        if version.len() == 0 {
                            version = item.title().unwrap_or_default();
                        }
                        let result = RemoteInfo {
                            version: version.to_string(),
                            update_page_url: enclosure.url.to_string()
                        };
                        return result;
                    }
                }
            }
        }
    }
    RemoteInfo {
        version: "-1".to_string(),
        update_page_url: String::new()
    }
}

/// 通过 itunes api 查询应用信息
#[tokio::main]
async fn mas_app_check(area_code: &str, bundle_id: &str) -> Option<RemoteInfo> {
    if let Ok(resp) = reqwest::get(format!("https://itunes.apple.com/{}/lookup?bundleId={}", area_code, bundle_id)).await {
        if let Ok(text) = resp.text().await {
            let json_value: serde_json::Value = serde_json::from_str(&text).unwrap();
            let result_count = json_value.get("resultCount").unwrap().as_u64().unwrap_or_default();
            if result_count != 0 {
                let results = json_value.get("results").unwrap();
                let version = results.get(0).unwrap().get("version").unwrap().to_string().replace("\"", "");
                let update_page_url = results.get(0).unwrap().get("trackViewUrl").unwrap().to_string().replace("\"", "");
                return Some(RemoteInfo {
                    version,
                    update_page_url
                });
            } else {
                return None;
            }
        }
    }
    return Some(RemoteInfo {
        version: "-1".to_string(),
        update_page_url: "".to_string()
    });
}

////////////////////////////////////////////////////////////////////////////////
// 版本号排序
////////////////////////////////////////////////////////////////////////////////

/// 版本号比对
fn cmp_version(a: &str, b: &str, compare_len: bool) -> cmp::Ordering {
    let mut a_version_str = a;
    if a.contains(" ") {
        let temp: Vec<&str> = a.split(" ").collect();
        a_version_str = temp.get(0).unwrap_or(&"");
    }
    let mut b_version_str = b;
    if b.contains(" ") {
        let temp: Vec<&str> = b.split(" ").collect();
        b_version_str = temp.get(0).unwrap_or(&"");
    }
    let arr1: Vec<&str> = a_version_str.split(".").collect();
    let arr2: Vec<&str> = b_version_str.split(".").collect();
    let length = cmp::min(arr1.len(), arr2.len());
    for i in 0..length {
        let num1: usize = arr1.get(i).unwrap_or(&"0").parse().unwrap_or(0);
        let num2: usize = arr2.get(i).unwrap_or(&"0").parse().unwrap_or(0);
        let re = num1.cmp(&num2);
        if re.is_eq() == false {
            return re;
        }
    }
    if compare_len {
        return arr1.len().cmp(&arr2.len());
    } else {
        return cmp::Ordering::Equal;
    }
}

////////////////////////////////////////////////////////////////////////////////
// 结构体和枚举
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct AppInfo {
    name: String,
    version: String,
    check_update_type: CheckUpType,
}

#[derive(Debug)]
enum CheckUpType {
    MAS(String),
    // iOS(String),
    Sparkle(String),
    HomeBrew {app_name: String, bundle_id: String}
}

struct InfoPlistInfo {
    version: String, 
    bundle_id: String,
    feed_url: Option<String>
}

struct RemoteInfo {
    version: String,
    update_page_url: String,
}