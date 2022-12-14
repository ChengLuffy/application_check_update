use std::{fs, path::Path, collections::HashMap};
use lazy_static::lazy_static;
use local::{AppInfo, CheckUpType};
use request::RemoteInfo;
use threadpool::ThreadPool;
use version_cmp::cmp_version;

pub mod local;
pub mod request;
pub mod version_cmp;

lazy_static! {
  static ref IGNORES: Vec<String> = local::config::get_ignore_config();
  static ref ALIAS: HashMap<String, String> = local::config::get_alias_config();
  static ref MASAREAS: Vec<String> = local::config::get_mas_areas();
  static ref SYSTEM_NAME: String = local::plist::get_system_version();
  static ref THREADNUMS: usize = local::config::get_thread_nums();
}

/// 检查指定路径下的应用
pub fn check_some(paths: Vec<&str>) {
  for item in paths {
      let path = Path::new(item);
      let buf = path.to_path_buf();
      if let Some(app_info) = local::check_app_info(&buf) {
          check_update(app_info)
      }
  }
}

/// 检查所有应用
pub fn check_all() {
  let apps_path = Path::new("/Applications");
  let n_workers: usize = *THREADNUMS;
  let pool = ThreadPool::new(n_workers);
  for item in fs::read_dir(apps_path).unwrap() {
      // 直接使用 thread::spawn 会产生 `Too many open files` 的问题，也不知道这是不是合适的解决方法
      pool.execute(move|| {
          match item {
              Ok(path) => {
                  let app_info = local::check_app_info(&path.path());
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
          CheckUpType::Mas {bundle_id, is_ios_app} =>  request::area_check(bundle_id, *is_ios_app), 
          CheckUpType::Sparkle(feed_url) => request::sparkle_feed_check(feed_url),
          CheckUpType::HomeBrew {app_name, bundle_id} => request::homebrew_check(app_name, bundle_id)
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