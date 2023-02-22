use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};

use crate::{local::check_app_info, IGNORES};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub threads_num: usize,
    pub mas_area: Vec<String>,
    pub alias: HashMap<String, String>,
    pub ignore: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        let ignore = vec![
            "com.apple.Safari".to_string(),
            "com.apple.SafariTechnologyPreview".to_string(),
            "org.gpgtools.gpgkeychain".to_string(),
        ];
        Self {
            threads_num: 5,
            mas_area: Default::default(),
            alias: Default::default(),
            ignore,
        }
    }
}

/// 设置应用别名
pub fn alias(bundle_id: &str, alias_name: &str) {
    let mut config = get_config().unwrap_or_default();
    if let Some(x) = config.alias.get_mut(bundle_id) {
        *x = alias_name.to_string();
    } else {
        config
            .alias
            .insert(bundle_id.to_string(), alias_name.to_string());
    }
    write_config(config);
    println!("Done!")
}

/// 忽略一些应用
pub fn ignore_some(bundle_id_vec: Vec<String>) {
    let mut config = get_config().unwrap_or_default();
    for item in bundle_id_vec {
        if let Some(app_info) = check_app_info(std::path::Path::new(&item)) {
            if !check_is_ignore(&app_info.bundle_id) {
                config.ignore.push(app_info.bundle_id)
            }
        }
    }
    write_config(config);
    println!("Done!")
}

/// 写入配置文件
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
pub async fn generate_config() {
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
              if input_string.to_lowercase().trim() == "y" {
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
        let content = fs::read_to_string(path).expect("读取配置文件时发生错误，`~/.config/appcu/config.yaml` 路径下不存在配置文件，您可以使用 `appcu generate_config` 生成一份默认配置文件");
        let config: Config =
            serde_yaml::from_str(&content).expect("解析配置文件时发生错误，配置文件格式错误");
        Some(config)
    } else {
        println!("未查到配置文件，您可以使用 `appcu generate_config` 生成一份默认配置文件");
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

/// 获取配置文件中设置的并发查询数量
pub fn get_thread_nums() -> usize {
    let conf = get_config();
    conf.unwrap_or_default().threads_num
}
