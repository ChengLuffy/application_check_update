use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ffi::OsString, fs};

use crate::{local::check_app_info, IGNORES};

/// 配置文件结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// 并发查询数
    pub threads_num: usize,
    /// terminal-notifier 安装路径
    pub terminal_notifier_path: String,
    /// Mac App Store 备用查询区域集合
    pub mas_area: Vec<String>,
    /// Homebrew 查询别名映射
    pub alias: HashMap<String, String>,
    /// 忽略查询应用集合
    pub ignore: Vec<String>,
}

impl Config {
    /// 写入配置文件
    fn write_to_file(&self) {
        let config_content = serde_yaml::to_string(self).expect("配置转换为文本错误");
        // 没有缩进感觉不对，希望这么改不会出问题
        let fmt_content = config_content.replace("\n-", "\n  -");
        let mut path = dirs::home_dir().expect("未能定位到用户目录");
        path.push(".config/appcu/config.yaml");
        fs::write(path, fmt_content).expect("写入配置文件失败");
    }
}

/// 实现 default trait
impl Default for Config {
    fn default() -> Self {
        let ignore = vec![
            "com.apple.Safari".to_string(),
            "com.apple.SafariTechnologyPreview".to_string(),
            "org.gpgtools.gpgkeychain".to_string(),
        ];
        Self {
            threads_num: 5,
            terminal_notifier_path: Default::default(),
            mas_area: Default::default(),
            alias: Default::default(),
            ignore,
        }
    }
}

/// 设置应用别名
pub fn alias(app_path: OsString, alias_name: OsString) {
    if let Some(app_info) = check_app_info(std::path::Path::new(&app_path)) {
        if let Ok(alias_name) = alias_name.into_string() {
            let bundle_id = app_info.bundle_id;
            let mut config = get_config().unwrap_or_default();
            if let Some(x) = config.alias.get_mut(&bundle_id) {
                *x = alias_name;
            } else {
                config.alias.insert(bundle_id.to_string(), alias_name);
            }
            config.write_to_file();
            println!("Done!")
        } else {
            println!("输入的 alias_name 读取失败")
        }
    } else {
        println!("读取应用信息失败")
    }
}

/// 忽略一些应用
pub fn ignore_some(bundle_id_vec: Vec<OsString>) {
    let mut config = get_config().unwrap_or_default();
    for item in bundle_id_vec {
        if let Some(app_info) = check_app_info(std::path::Path::new(&item)) {
            if !check_is_ignore(&app_info.bundle_id) {
                config.ignore.push(app_info.bundle_id)
            }
        }
    }
    config.write_to_file();
    println!("Done!")
}

/// 生成配置文件
pub fn generate_config() {
    let mut default_config = Config::default();
    default_config.alias.insert(
        "com.jetbrains.intellij.ce".to_string(),
        "intellij-idea-ce".to_string(),
    );
    // 在生成默认配置文件时尝试设置 terminal-notifier 的地址
    let mut terminal_notifier_path: String = "".to_string();
    if let Ok(output) = std::process::Command::new("which")
        .arg("terminal-notifier")
        .output()
    {
        let stdout = output.stdout;
        if !stdout.is_empty() {
            let path_string = String::from_utf8_lossy(&stdout).to_string();
            terminal_notifier_path = path_string.trim().to_string();
        }
    }
    default_config.terminal_notifier_path = terminal_notifier_path;
    let config_content = serde_yaml::to_string(&default_config).expect("配置转换为文本错误");
    let fmt_content = config_content.replace("\n-", "\n  -");
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
        if input_string.to_lowercase().trim() == "y" {
            let start = std::time::SystemTime::now();
            let since_the_epoch = start
                .duration_since(std::time::UNIX_EPOCH)
                .expect("时间戳获取失败");
            let ms = since_the_epoch.as_secs() as i64 * 1000i64
                + (since_the_epoch.subsec_nanos() as f64 / 1_000_000.0) as i64;
            let mut new_path = dirs::home_dir().unwrap();
            let new_name = format!(".config/appcu/config.yaml_bk_{ms}");
            new_path.push(new_name);
            fs::rename(&path, new_path).expect("原有配置文件重命名错误");
        } else {
            println!("用户取消默认配置文件生成");
            return;
        }
    }
    fs::write(path, fmt_content).expect("配置文件写入错误")
}

////////////////////////////////////////////////////////////////////////////////
// 获取配置信息
////////////////////////////////////////////////////////////////////////////////

/// 获取配置信息
///
/// - 配置文件，使用 `bundle id` 确定相应的应用，两种使用场景
/// - 1. 忽略应用，比如企业证书分发的应用，还有无法通过应用商店、Sparkle方式、HomeBrew-Casks 查询到应用版本信息的应用，或者不想检查更新的应用；
/// - 2. HomeBrew-Casks 检测时的别名，大部分应用需要配置
fn get_config() -> Option<Config> {
    let mut path = dirs::home_dir().expect("未能定位到用户目录");
    path.push(".config/appcu/config.yaml");
    if path.exists() {
        let content = fs::read_to_string(path).expect("读取配置文件时发生错误，`~/.config/appcu/config.yaml` 路径下不存在配置文件，您可以使用 `appcu generate-config` 生成一份默认配置文件");
        let config: Config =
            serde_yaml::from_str(&content).expect("解析配置文件时发生错误，配置文件格式错误");
        Some(config)
    } else {
        None
    }
}

/// 获取别名配置
pub fn get_alias_config() -> HashMap<String, String> {
    let conf = get_config();
    conf.unwrap_or_default().alias
}

/// 获取忽略配置
pub fn get_ignore_config() -> Vec<String> {
    let conf = get_config();
    conf.unwrap_or_default().ignore
}

/// 查询是否是忽略应用
pub fn check_is_ignore(bundle_id: &str) -> bool {
    let arr = IGNORES.to_vec();
    arr.iter().any(|x| x == bundle_id)
}

/// 获取配置文件中备用的商店区域代码
pub fn get_mas_areas() -> Vec<String> {
    let conf = get_config();
    conf.unwrap_or_default().mas_area
}

/// 获取配置文件中设置的 terminal-notifier 的安装路径
pub fn get_terminal_notifier_path() -> String {
    let conf = get_config();
    conf.unwrap_or_default().terminal_notifier_path
}

/// 获取配置文件中设置的并发查询数量
pub fn get_thread_nums() -> usize {
    let conf = get_config();
    conf.unwrap_or_default().threads_num
}
