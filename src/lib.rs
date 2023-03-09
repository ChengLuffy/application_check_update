use lazy_static::lazy_static;
use local::notification::Notification;
use local::{AppInfo, CheckUpType};
use request::RemoteInfo;
use std::{collections::HashMap, fs, path::Path};
use threadpool::ThreadPool;
use version_cmp::cmp_version;

pub mod local;
pub mod request;
pub mod version_cmp;

lazy_static! {
    static ref IGNORES: Vec<String> = local::config::get_ignore_config();
    static ref ALIAS: HashMap<String, String> = local::config::get_alias_config();
    static ref MASAREAS: Vec<String> = local::config::get_mas_areas();
    static ref ARM_SYSTEM_NAME: String = local::plist::get_arm_system_version();
    static ref THREADNUMS: usize = local::config::get_thread_nums();
}

#[derive(Copy, Clone)]
pub struct CheckOperation {
    pub notification: bool,
    pub verbose: bool,
}

impl CheckOperation {
    /// 检查指定路径下的应用
    pub fn check_some(&self, paths: Vec<String>) {
        for item in paths {
            let path = Path::new(&item);
            let buf = path.to_path_buf();
            if let Some(app_info) = local::check_app_info(&buf) {
                self.check_update(app_info)
            } else if self.notification {
                Notification::new_error_notification(format!("{item} 应用信息读取失败")).post()
            } else {
                println!("+++++");
                println!("{item} 应用信息读取失败");
                println!("+++++\n");
            }
        }
    }

    /// 检查所有应用
    pub fn check_all(&self) {
        let temp_self = *self;
        let apps_path = Path::new("/Applications");
        let n_workers: usize = *THREADNUMS;
        let pool = ThreadPool::new(n_workers);
        for item in fs::read_dir(apps_path).unwrap() {
            // 直接使用 thread::spawn 会产生 `Too many open files` 的问题
            pool.execute(move || match item {
                Ok(path) => {
                    let app_info = local::check_app_info(&path.path());
                    if let Some(info) = app_info {
                        temp_self.check_update(info);
                    }
                }
                Err(error) => {
                    println!("+++++");
                    println!("{error:?}");
                    println!("+++++\n");
                    Notification::new_error_notification(format!("{error:?}")).post()
                }
            });
        }
        pool.join();
    }

    /// 根据应用类型查询更新并输出
    pub fn check_update(&self, app_info: AppInfo) {
        let check_update_type = &app_info.check_update_type;
        let mut remote_info: RemoteInfo = RemoteInfo {
            version: "-1".to_string(),
            update_page_url: "".to_string(),
        };
        for _ in 0..5 {
            remote_info = match check_update_type {
                CheckUpType::Mas {
                    bundle_id,
                    is_ios_app,
                } => request::area_check(bundle_id, *is_ios_app),
                CheckUpType::Sparkle(feed_url) => request::sparkle_feed_check(feed_url),
                CheckUpType::HomeBrew {
                    app_name,
                    bundle_id,
                } => request::homebrew_check(app_name, bundle_id),
                // _ => RemoteInfo { version: "-2".to_string(), update_page_url: String::new() }
            };
            if remote_info.version != *"-1" {
                break;
            }
        }
        if remote_info.version == *"-1" {
            if self.notification {
                Notification::new_remote_get_failed(&app_info).post()
            } else {
                println!("+++++");
                println!("{}", app_info.name);
                println!("{}", app_info.check_update_type);
                println!("local version {}", app_info.short_version);
                println!("remote version check failed");
                println!("+++++\n");
            }
        }
        // FIXME: 丑陋的代码，这一段代码变成这样的原因，Sparkle 应用各有各的写法，有的应用只有从 title 读取版本号，有的从 item 有的从 enclosure
        // FIXME: 版本号也有问题，有的 sparkle:version 是 x.x.x 的形式，有的 sparkle:shortVersionString 是
        // FIXME: homebrew 的接口也有点问题，比如 Version 是 4.0，通过接口查询会变成 4，比如有些应用本地查到是 7.0.2，接口查到是 7.0.2.7，但其实是一个版本
        let local_cmp_version = if !app_info.short_version.is_empty()
            && !matches!(app_info.check_update_type, CheckUpType::Sparkle(_))
            || (remote_info.version.contains('.') && app_info.short_version.contains('.'))
        {
            &app_info.short_version
        } else {
            &app_info.version
        };
        let ordering = cmp_version(local_cmp_version, &remote_info.version, false);
        if ordering.is_lt() {
            // if &remote_info.version != "-2" {
            if self.notification {
                Notification::new_update_notification(
                    app_info.name,
                    local_cmp_version.to_string(),
                    remote_info.version,
                    remote_info.update_page_url,
                )
                .post()
            } else {
                println!("=====");
                println!("{}", app_info.name);
                if self.verbose {
                    println!("{}", app_info.check_update_type);
                }
                println!("{local_cmp_version} -> {}", remote_info.version);
                println!("{}", remote_info.update_page_url);
                println!("=====\n");
            }
        } else if self.verbose {
            if self.notification {
                Notification::new_verbose_notification(
                    &app_info,
                    local_cmp_version.to_string(),
                    &remote_info,
                )
                .post()
            } else {
                println!("-----");
                println!("{}", app_info.name);
                println!("{}", app_info.check_update_type);
                println!("local version {local_cmp_version}");
                println!("remote version {}", remote_info.version);
                println!("{}", remote_info.update_page_url);
                println!("-----\n");
            }
        }
    }
}
