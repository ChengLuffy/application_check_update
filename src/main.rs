use core::str;
use std::{fs, path::{Path, PathBuf}, cmp, collections::HashMap, fmt::Display};
use plist::Value;
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};
use skyscraper::html;
use threadpool::ThreadPool;
use rss::Channel;
use std::ffi::OsString;
use clap::Command;

lazy_static! {
    static ref IGNORES: Vec<String> = get_ignore_config();
    static ref ALIAS: HashMap<String, String> = get_alias_config();
    static ref MASAREAS: Vec<String> = get_mas_areas();
    static ref SYSTEM_NAME: String = get_system_version();
    static ref THREADNUMS: usize = get_thread_nums();
}

/// TODO: 尝试使用 tui 输出 （是否要做，挺麻烦的）？
fn main() {
    let command = Command::new("appcu")
                            .name("appcu")
                            .about("macOS 应用检查更新")
                            .allow_external_subcommands(true)
                            .override_usage("\n  运行 `appcu` 对所有 `/Applications` 文件夹下的应用进行检查；\n  运行 `appcu /Applications/xx.app /Applications/yy.app` 对特定应用进行检查；")
                            .subcommand(Command::new("ignore")
                                                        .about("忽略对应的应用")
                                                        .override_usage("appcu ignore /Applications/xx.app /Applications/yy.app"))
                            .subcommand(Command::new("alias")
                                                        .about("设置 HomeBrew 查询方式的应用别名")
                                                        .override_usage("appcu alias app.bundle.id alias_name"))
                            .subcommand(Command::new("generate_config")
                                                        .alias("gc")
                                                        .about("简写: `gc`，生成配置文件，详情请查看 `appcu help generate_config`")
                                                        .override_usage("appcu generate_config\n\n配置文件路径：`~/.config/appcu/config.yaml`\n配置文件说明：\n```\n# 并行查询数量，默认 5，太多会导致错误\nthreads_num: 5\n\n# 用于 App Store 备选区域查询，默认是当前登陆 Mac App Store 的账号区域，如果有一些应用是其他区域专属，可以在此添加\nmas_area:\n  # 例如我的主账号为美区账号，但是一些应用使用国区账号下载，所以将国区 `area_code` 添加在此处\n  - cn\n\n# HomeBrew 查询时，是将应用名称直接查寻，但是某些应用无法直接查到，可以在这里设置查询的别名\nalias:\n  # 例如：wps，获取当前安装的应用 bundle_id 为 `com.kingsoft.wpsoffice.mac`\n  # 通过 `brew search wpsoffice` 选择 `wpsoffice-cn`，映射如下\n  com.kingsoft.wpsoffice.mac: wpsoffice-cn\n\n# 有些应用不用查询，或者无法查询（例如已经下架、未被收录在 HomeBrew 等），可以在这里设置忽略\nignore:\n  # 例 safari 无法通过任何手段查询更新，获取 safari bundle_id 进行忽略\n  # 也可以利用 `appcu ignore ...` 进行忽略\n  - com.apple.Safari\n```"))
                            .version("0.1.0");
    let args = command.get_matches();
    if let Some((external, ext_m)) = args.subcommand() {
        let mut ext_args: Vec<&str> = ext_m.get_many::<OsString>("").unwrap_or_default().map(|x| x.to_str().unwrap_or_default()).collect();
        let mut results = vec![external];
        results.append(&mut ext_args);
        if results.is_empty() {
            check_all()
        } else if results.len() == 1 && external == "generate_config" {
            generate_config()
        } else if external.starts_with("ignore") { // FIXME: 好像这么写有问题，但还不确定怎么改
            if !external.starts_with("ignore ") {
                println!("未能识别 ignore 参数，ignore 命令使用方式为 `appcu ignore /Applications/xx.app/ /Applications/yy.app/ ...`")
            } else {
                let mut vec: Vec<&str> = external.split(' ').collect();
                vec.remove(0);
                if vec.is_empty() {
                    println!("未能识别 ignore 参数，ignore 命令使用方式为 `appcu ignore /Applications/xx.app/ /Applications/yy.app/ ...`")
                } else {
                    ignore_some(vec)
                }
            }
        } else if external.starts_with("alias") { // FIXME: 好像这么写有问题，但还不确定怎么改
            if !external.starts_with("alias ") {
                println!("未能识别 bundle_id 参数，alias 命令使用方式为 `appcu alias app.bundle.id alias_name`") 
            } else {
                let mut vec: Vec<&str> = external.split(' ').collect();
                vec.remove(0);
                if vec.len() == 2 {
                    let bundle_id = vec.first().unwrap();
                    if bundle_id.contains('.') {
                        let alias_name = vec.get(1).unwrap();
                        alias(bundle_id, alias_name)
                    } else {
                        println!("未能识别 bundle_id 参数，alias 命令使用方式为 `appcu alias app.bundle.id alias_name`")
                    }
                } else {
                    println!("未能识别命令，alias 命令使用方式为 `appcu alias app.bundle.id alias_name`")
                }
            }
        } else {
            check_some(results)
        }
    } else {
        check_all()
    }
}

fn alias(bundle_id: &str, alias_name: &str) {
    let mut config = get_config();
    if let Some(x) = config.alias.get_mut(bundle_id) {
        *x = alias_name.to_string();
    } else {
        config.alias.insert(bundle_id.to_string(), alias_name.to_string());
    }
    // println!("{:?}", config.alias);
    write_config(config);
    println!("设置成功")
}

/// 忽略一些应用
fn ignore_some(bundle_id_vec: Vec<&str>) {
    let mut config = get_config();
    for item in bundle_id_vec {
        if let Some(app_info) = check_app_info(Path::new(item)) {
            if !check_is_ignore(&app_info.bundle_id) {
                config.ignore.push(app_info.bundle_id)
            }
        }
    }
    write_config(config);
    println!("设置成功")
}

fn write_config(config: Config) {
    let config_content = serde_yaml::to_string(&config).expect("配置转换为文本错误");
    // 没有缩进感觉不对，希望这么改不会出问题
    let fmt_content = config_content.replace("\n-", "\n  -");
    let mut path = dirs::home_dir().expect("未能定位到用户目录");
    path.push(".config/appcu/config.yaml");
    fs::write(path, fmt_content).expect("写入配置文件失败");
}

/// 生成配置文件
#[tokio::main]
async fn generate_config() {
    if let Ok(content) = reqwest::get("https://raw.githubusercontent.com/ChengLuffy/application_check_update/master/default_config.yaml").await {
        if let Ok(text_content) = content.text().await {
            let mut path = dirs::home_dir().expect("未能定位到用户目录");
            path.push(".config/appcu");
            if !path.exists() {
                fs::create_dir_all(&path).expect("创建文件夹错误");
            }
            path.push("config.yaml");
            if path.exists() {
                let mut input_string = String::new();
                println!("已经存在一份配置文件，继续运行会将现有的配置文件重命名并生成一份默认配置文件，是否继续？：(y or ...) ");
                std::io::stdin().read_line(&mut input_string).unwrap();
                if input_string.to_lowercase() == "y" {
                    let start = std::time::SystemTime::now();
                    let since_the_epoch = start
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .expect("时间戳获取失败");
                    let ms = since_the_epoch.as_secs() as i64 * 1000i64 + (since_the_epoch.subsec_nanos() as f64 / 1_000_000.0) as i64;
                    let mut new_path = dirs::home_dir().unwrap();
                    let new_name = format!(".config/appcu/config.yaml_bk_{ms}");
                    new_path.push(new_name);
                    fs::rename(&path, new_path).expect("原有配置文件重命名错误");
                } else {
                    println!("用户取消默认配置文件生成");
                    return;
                }
            }
            fs::write(path, text_content).expect("配置文件写入错误")
        } else {
            println!("默认配置解码失败")
        }
    } else {
        println!("获取默认配置失败")
    }
}

/// 检查指定路径下的应用
fn check_some(paths: Vec<&str>) {
    for item in paths {
        let path = Path::new(item);
        let buf = path.to_path_buf();
        if let Some(app_info) = check_app_info(&buf) {
            check_update(app_info)
        }
    }
}

/// 检查所有应用
fn check_all() {
    let apps_path = Path::new("/Applications");
    let n_workers: usize = *THREADNUMS;
    let pool = ThreadPool::new(n_workers);
    for item in fs::read_dir(apps_path).unwrap() {
        // 直接使用 thread::spawn 会产生 `Too many open files` 的问题，也不知道这是不是合适的解决方法
        pool.execute(move|| {
            match item {
                Ok(path) => {
                    let app_info = check_app_info(&path.path());
                    if let Some(info) = app_info {
                        check_update(info);
                    }
                },
                Err(error) => println!("{:?}", error)
            }
        });
    }
    pool.join();
}

/// 根据应用类型查询更新并输出
fn check_update(app_info: AppInfo) {
    let check_update_type = &app_info.check_update_type;
    let mut remote_info: RemoteInfo;
    loop {
        remote_info = match check_update_type {
            CheckUpType::Mas {bundle_id, is_ios_app} =>  area_check(bundle_id, *is_ios_app), 
            CheckUpType::Sparkle(feed_url) => sparkle_feed_check(feed_url),
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
    if remote_info.version.is_empty() {
        println!("+++++");
        println!("{}", app_info.name);
        println!("{}", app_info.check_update_type);
        println!("local version {}", app_info.short_version);
        println!("remote version check failed");
        println!("+++++\n");
    }
    // FIXME: 丑陋的代码，这一段代码变成这样的原因，Sparkle 应用各有各的写法，有的应用只有从 title 读取版本号，有的从 item 有的从 enclosure，版本好也有问题，有的 sparkle:version 是 x.x.x 的形式，有的 sparkle:shortVersionString 是，homebrew 的接口也有点问题，比如 Version 是 4.0，通过接口查询会变成 4
    let local_cmp_version = if !app_info.short_version.is_empty() && !matches!(app_info.check_update_type, CheckUpType::Sparkle(_)) || ( remote_info.version.contains('.') && app_info.short_version.contains('.')) {
        &app_info.short_version
    } else {
        &app_info.version
    };
    let ordering = cmp_version(local_cmp_version, &remote_info.version, false);
    if ordering.is_lt() {
    // if &remote_info.version != "-2" {
        println!("=====");
        println!("{}", app_info.name);
        println!("local version {}", local_cmp_version);
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
fn check_app_info(entry: &Path) -> Option<AppInfo> {
    let path = entry;
    let app_name = path.file_name().unwrap_or_default();
    let app_name_str = app_name.to_str().unwrap_or_default();
    if !app_name_str.starts_with('.') && app_name_str.ends_with(".app") {
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
            let cu_type = CheckUpType::Mas {
                bundle_id: plist_info.bundle_id.to_string(),
                is_ios_app: true
            };
            let app_info = AppInfo {
                name: name_str.to_string(),
                version: plist_info.version,
                short_version: plist_info.short_version,
                bundle_id: plist_info.bundle_id,
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
                cu_type = CheckUpType::Mas {
                    bundle_id: plist_info.bundle_id.to_string(),
                    is_ios_app: false
                };
            } else if let Some(feed_url) = plist_info.feed_url {
                cu_type = CheckUpType::Sparkle(feed_url);
            } else {
                cu_type = CheckUpType::HomeBrew {
                    app_name: name_str.to_string(),
                    bundle_id: plist_info.bundle_id.replace(':', "")
                };
            }
            let app_info = AppInfo {
                name: name_str.to_string(), 
                version: plist_info.version.to_string(), 
                short_version: plist_info.short_version.to_string(),
                bundle_id: plist_info.bundle_id.to_string(),
                check_update_type: cu_type
            };
            return Some(app_info);
        }
    }
    None
}

/// 从 `Info.plist` 文件中读取有用信息
fn read_plist_info(plist_path: &PathBuf) -> InfoPlistInfo {
    let mut short_version_key_str = "CFBundleShortVersionString";
    let mut version_key_str = "CFBundleVersion";
    let mut bundle_id_key_str = "CFBundleIdentifier";
    let feed_url_key = "SUFeedURL";
    if !plist_path.ends_with("Info.plist") {
        short_version_key_str = "bundleShortVersionString";
        version_key_str = "bundleVersion";
        bundle_id_key_str = "softwareVersionBundleId";
    }
    let value = Value::from_file(plist_path).expect("failed to read plist file");
    let bundle_id = value
                            .as_dictionary()
                            .and_then(|dict| dict.get(bundle_id_key_str))
                            .and_then(|id| id.as_string()).unwrap_or("");
    if bundle_id.is_empty() {
        let info_plist_path = plist_path.parent().unwrap().parent().unwrap().join("WrappedBundle/Info.plist");
        return read_plist_info(&info_plist_path);
    }
    let version = value
                                .as_dictionary()
                                .and_then(|dict| dict.get(version_key_str))
                                .and_then(|id| id.as_string()).unwrap_or("");
    let short_version = value
                                .as_dictionary()
                                .and_then(|dict| dict.get(short_version_key_str))
                                .and_then(|id| id.as_string()).unwrap_or("");
    let feed_url_option = value
                    .as_dictionary()
                    .and_then(|dict| dict.get(feed_url_key))
                    .and_then(|id| id.as_string());
    let feed_url = feed_url_option.map(|string| string.to_string());
    InfoPlistInfo {version: version.to_string(), short_version: short_version.to_string(), bundle_id: bundle_id.to_string(), feed_url}
}

////////////////////////////////////////////////////////////////////////////////
// 获取配置信息
////////////////////////////////////////////////////////////////////////////////

/// 获取配置信息
/// 
/// - 配置文件，使用 `bundle id` 确定相应的应用，两种使用场景
/// - 1. 忽略应用，比如企业证书分发的应用，还有无法通过应用商店、Sparkle方式、HomeBrew-Casks 查询到应用版本信息的应用，或者不想检查更新的应用；
/// - 2. HomeBrew-Casks 检测时的别名，大部分应用需要配置
fn get_config() -> Config {
    let mut path = dirs::home_dir().expect("未能定位到用户目录");
    path.push(".config/appcu/config.yaml");
    let content = fs::read_to_string(path).expect("读取配置文件时发生错误，`~/.config/appcu/config.yaml` 路径下不存在配置文件，您可以使用 `appcu generate_config` 生成一份默认配置文件");
    let config: Config = serde_yaml::from_str(&content).expect("解析配置文件时发生错误，配置文件格式错误");
    config
}

/// 获取别名配置
fn get_alias_config() -> HashMap<String, String> {
    let conf = get_config();
    conf.alias
}

/// 获取忽略配置
fn get_ignore_config() -> Vec<String> {
    let conf = get_config();
    conf.ignore
}

/// 查询是否是忽略应用
fn check_is_ignore(bundle_id: &str) -> bool {
    let arr = IGNORES.to_vec();
    arr.iter().any(|x| x == bundle_id)
}

/// 获取配置文件中备用的商店区域代码
fn get_mas_areas() -> Vec<String> {
    let conf = get_config();
    conf.mas_area
}

/// 获取配置文件中设置的并发查询数量
fn get_thread_nums() -> usize {
    let conf = get_config();
    conf.threads_num
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
    "".to_string()
}

////////////////////////////////////////////////////////////////////////////////
// 网络请求
////////////////////////////////////////////////////////////////////////////////

/// 查询 HomeBrew-Casks 内的版本信息
/// 
/// 读取 `Homebrew/homebrew-cask` 仓库 `Casks` 文件夹内的响应应用文件
#[tokio::main]
async fn homebrew_check(app_name: &str, bundle_id: &str) -> RemoteInfo {
    let dealed_app_name = app_name.to_lowercase().replace(' ', "-");
    let alias_keys = ALIAS.keys();
    let file_name = if !alias_keys.into_iter().any(|x| x == &bundle_id.to_string()) {
        &dealed_app_name
    } else {
        &ALIAS[bundle_id]
    };
    if let Ok(resp) = reqwest::get(format!("https://formulae.brew.sh/api/cask/{}.json", file_name)).await {
        if let Ok(text) = resp.text().await {
            if let Ok(json_value) = serde_json::from_str(&text) {
                let json_value: serde_json::Value = json_value;
                let version_arr: Vec<&str> = json_value.get("version").unwrap().as_str().unwrap().split(',').collect();
                let version: &str = version_arr.first().unwrap_or(&"");
                let arch_str = std::env::consts::ARCH;
                let mut url = json_value.get("url").unwrap().as_str().unwrap_or_default().to_string();
                if SYSTEM_NAME.len() > 0 && (arch_str == "aarch64" || arch_str == "arm") {
                    if let Some(variations) = json_value.get("variations") {
                        if let Some(arm64_ventura) = variations.get(SYSTEM_NAME.as_str()) {
                            if let Some(url_value) = arm64_ventura.get("url") {
                                let url_temp = url_value.as_str().unwrap_or_default().to_string();
                                url = url_temp;
                            }
                        }
                    }
                }
                return RemoteInfo {
                    version: version.to_string(),
                    update_page_url: url
                };
            } else {
                return RemoteInfo {
                    version: "".to_string(),
                    update_page_url: String::new()
                };
            }
        }
    }
    RemoteInfo {
        version: "-1".to_string(),
        update_page_url: String::new()
    }
}

#[tokio::main]
async fn sparkle_feed_check(feed_url: &str) -> RemoteInfo {
    if let Ok(content) = reqwest::get(feed_url).await {
        if let Ok(bytes_content) = content.bytes().await {
            if let Ok(channel) = Channel::read_from(&bytes_content[..]) {
                let mut items: Vec<rss::Item> = channel.items().into();
                // FIXME: 有些应用例如 playcover 2.0.1 版本，应用内 Info.plist 的版本却是 2.0.0 ，但是 shortVersion 又不是所有应用都有的
                // FIXME: xml 格式也不统一，有些把版本信息放在 enclosure 内，有些是直接是标题，有些是 item 内
                items.sort_by(|a, b| {
                    if let Some(a_enclosure) = a.enclosure() {
                        if let Some(b_enclosure) = b.enclosure() {
                            let mut a_version = a_enclosure.version.as_str();
                            if !a_version.contains('.') {
                                a_version = a_enclosure.short_version.as_str();
                            }
                            if a_version.is_empty() {
                                a_version = a.title().unwrap_or_default();
                            }
                            let mut b_version = b_enclosure.version.as_str();
                            if !b_version.contains('.') {
                                b_version = b_enclosure.short_version.as_str();
                            }
                            if b_version.is_empty() {
                                b_version = b.title().unwrap_or_default();
                            }
                            return cmp_version(a_version, b_version, true);
                        }
                    }
                    std::cmp::Ordering::Equal
                });
                if let Some(item) = items.last() {
                    let mut version = item.version().unwrap_or_default();
                    if version.is_empty() {
                        version = item.short_version().unwrap_or_default();
                    }
                    if version.is_empty() {
                        if let Some(enclosure) = item.enclosure() {
                            let mut version = enclosure.version.as_str();
                            if !version.contains('.') {
                                version = &enclosure.short_version;
                            }
                            if version.is_empty() {
                                version = item.title().unwrap_or_default();
                            }
                            let result = RemoteInfo {
                                version: version.to_string(),
                                update_page_url: enclosure.url.to_string()
                            };
                            return result;
                        }
                    }
                    let result = RemoteInfo {
                        version: version.to_string(),
                        update_page_url: item.enclosure().unwrap().url.to_string()
                    };
                    return result;
                }
            }
        }
    }
    RemoteInfo {
        version: "-1".to_string(),
        update_page_url: String::new()
    }
}

/// MAS 应用和 iOS 应用可能存在区域内未上架的问题，采取先检测 cn 后检测 us 的方式
fn area_check(bundle_id: &str, is_ios_app: bool) -> RemoteInfo {
    let mut mas_areas = MASAREAS.to_vec();
    mas_areas.insert(0, "".to_string());
    for area_code in mas_areas {
        let remote_info_opt = mas_app_check(&area_code, bundle_id, is_ios_app);
        if let Some(remote_info) = remote_info_opt {
            return remote_info;
        }
    }
    RemoteInfo {version: String::new(), update_page_url: "".to_string()}
}

/// 通过 itunes api 查询应用信息
#[tokio::main]
async fn mas_app_check(area_code: &str, bundle_id: &str, is_ios_app: bool) -> Option<RemoteInfo> {
    if let Ok(resp) = reqwest::get(format!("https://itunes.apple.com/{}/lookup?bundleId={}", area_code, bundle_id)).await {
        if let Ok(text) = resp.text().await {
            let json_value: serde_json::Value = serde_json::from_str(&text).unwrap();
            let result_count = json_value.get("resultCount").unwrap().as_u64().unwrap_or_default();
            if result_count != 0 {
                let results = json_value.get("results").unwrap();
                let item = results.get(0).unwrap();
                let update_page_url = item.get("trackViewUrl").unwrap().to_string().replace('\"', "");
                let mut version = item.get("version").unwrap().to_string().replace('\"', "");
                // iOS 和 iPadOS 的应用不需要走这个流程
                if !is_ios_app {
                    // FIXME: 某些 iOS 和 macOS 应用使用一样的 bundleid 现在的查询方法只会返回 iOS 的结果，例如：ServerCat PasteNow，暂时的解决方案：抓取网页数据，匹配 <p class="l-column small-6 medium-12 whats-new__latest__version">Version/版本 x.x.x</p>
                    // FIXME: 上述方案会偶发性查不到，原因是通过 trackViewUrl 获取的 html 文本可能是没查到信息前的 loading 文本，所以 loop 一下
                    // FIXME: 还有一种情况，例如 QQ 6.9.0 通过 iTunes api cn 可以查到 6.9.0 版本，但是 us 还是 6.8.9，所以统一改成再用应用主页查一遍
                    let client = reqwest::Client::new();
                    loop {
                        if let Ok(resp) = client.get(&update_page_url).header("USER_AGENT", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.2 Safari/605.1.15").send().await {
                            if let Ok(text) = resp.text().await {
                                if let Ok(document) = html::parse(&text) {
                                    let xpath = skyscraper::xpath::parse::parse("//p[@class='l-column small-6 medium-12 whats-new__latest__version']").unwrap();
                                    if let Ok(nodes) = xpath.apply(&document) {
                                        if let Some(doc_node) = nodes.get(0) {
                                            if let Some(text) = doc_node.get_text(&document) {
                                                if let Some(last) = text.split(' ').last() {
                                                    version = last.to_string();
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                return Some(RemoteInfo {
                    version,
                    update_page_url
                });
            } else {
                return None;
            }
        }
    }
    Some(RemoteInfo {
        version: "-1".to_string(),
        update_page_url: "".to_string()
    })
}

////////////////////////////////////////////////////////////////////////////////
// 版本号排序
////////////////////////////////////////////////////////////////////////////////

/// 版本号比对
fn cmp_version(a: &str, b: &str, compare_len: bool) -> cmp::Ordering {
    let mut a_version_str = a;
    if a.contains(' ') {
        let temp: Vec<&str> = a.split(' ').collect();
        a_version_str = temp.first().unwrap_or(&"");
    }
    let mut b_version_str = b;
    if b.contains(' ') {
        let temp: Vec<&str> = b.split(' ').collect();
        b_version_str = temp.first().unwrap_or(&"");
    }
    let arr1: Vec<&str> = a_version_str.split('.').collect();
    let arr2: Vec<&str> = b_version_str.split('.').collect();
    let length = cmp::min(arr1.len(), arr2.len());
    for i in 0..length {
        let num1: usize = arr1.get(i).unwrap_or(&"0").parse().unwrap_or(0);
        let num2: usize = arr2.get(i).unwrap_or(&"0").parse().unwrap_or(0);
        let re = num1.cmp(&num2);
        if !re.is_eq() {
            return re;
        }
    }
    if compare_len {
        arr1.len().cmp(&arr2.len())
    } else {
        cmp::Ordering::Equal
    }
}

////////////////////////////////////////////////////////////////////////////////
// 结构体和枚举
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct AppInfo {
    name: String,
    version: String,
    short_version: String,
    bundle_id: String,
    check_update_type: CheckUpType
}

#[derive(Debug, PartialEq)]
enum CheckUpType {
    Mas {bundle_id: String, is_ios_app: bool},
    Sparkle(String),
    HomeBrew {app_name: String, bundle_id: String}
}

impl Display for CheckUpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckUpType::Mas { bundle_id, is_ios_app } => {
                let temp = if *is_ios_app {
                    "，这是一个 iOS/iPadOS 应用"
                } else {
                    ""
                };
                write!(f, "检查更新方式为 iTunes API，获取到的 bundle_id 为: {}{}", bundle_id, temp)
            },
            CheckUpType::Sparkle(feed_url) => write!(f, "检查更新方式为 Sparkle，获取到的 SUFeedURL 为: {}", feed_url),
            CheckUpType::HomeBrew { app_name, bundle_id } => {
                let dealed_app_name = app_name.to_lowercase().replace(' ', "-");
                write!(f, "检查更新方式为 HomeBrew，获取到的 bundle_id 为: {}，默认的信息获取链接为: https://formulae.brew.sh/api/cask/{}.json", bundle_id, dealed_app_name)
            }
        }
    }
}

struct InfoPlistInfo {
    version: String, 
    short_version: String,
    bundle_id: String,
    feed_url: Option<String>
}

struct RemoteInfo {
    version: String,
    update_page_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    threads_num: usize,
    mas_area: Vec<String>,
    alias: HashMap<String, String>,
    ignore: Vec<String>
}