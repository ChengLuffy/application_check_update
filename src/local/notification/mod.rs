use crate::request::RemoteInfo;

use super::AppInfo;

pub struct Notification {
    pub title: String,
    pub subtitle: String,
    pub message: String,
    pub open: Option<String>,
}

impl Notification {
    pub fn new_error_notification(msg: String) -> Self {
        Notification {
            title: "❌".to_string(),
            subtitle: "appcu".to_string(),
            message: msg,
            open: None,
        }
    }
    pub fn new_remote_get_failed(app_info: &AppInfo) -> Self {
        Notification {
            title: format!("❌{} 获取最新版本失败", app_info.name),
            subtitle: "appcu".to_string(),
            message: format!("{}", app_info.check_update_type),
            open: None,
        }
    }
    pub fn new_update_notification(
        name: String,
        local_version: String,
        remote_version: String,
        update_page_url: String,
    ) -> Self {
        Notification {
            title: format!("🎉{} 有更新", name),
            subtitle: "点击通知下载最新版本安装包".to_string(),
            message: format!("{} -> {}", local_version, remote_version),
            open: Some(update_page_url),
        }
    }
    pub fn new_verbose_notification(
        app_info: &AppInfo,
        local_version: String,
        remote_info: &RemoteInfo,
    ) -> Self {
        Notification {
            title: format!("{} 检查结束", app_info.name),
            subtitle: "点击通知下载最新版本安装包".to_string(),
            message: format!(
                "{} -> {}\n{}",
                local_version, remote_info.version, app_info.check_update_type
            ),
            open: Some(remote_info.update_page_url.clone()),
        }
    }
    pub fn post(&self) {
        let output = match &self.open {
            Some(open) => std::process::Command::new("terminal-notifier")
                .arg("-title")
                .arg(&self.title)
                .arg("-subtitle")
                .arg(&self.subtitle)
                .arg("-message")
                .arg(&self.message)
                .arg("-open")
                .arg(open)
                .output(),
            None => std::process::Command::new("terminal-notifier")
                .arg("-title")
                .arg(&self.title)
                .arg("-subtitle")
                .arg(&self.subtitle)
                .arg("-message")
                .arg(&self.message)
                .output(),
        };
        match output {
            Ok(_) => {}
            Err(err) => match &err.kind() {
                std::io::ErrorKind::NotFound => println!("未能找到 `terminal-notifier`"),
                _ => println!("{:?}", err),
            },
        }
    }
}
