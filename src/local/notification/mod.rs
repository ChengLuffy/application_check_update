use super::AppInfo;
use crate::{request::RemoteInfo, TERMINAL_NOTIFIER_PATH};

#[derive(Debug)]
pub struct Notification {
    pub title: String,
    pub subtitle: String,
    pub message: String,
    pub open_url: Option<String>,
    pub open_by_app: bool,
}

impl Notification {
    pub fn new_error_notification(msg: String) -> Self {
        Notification {
            title: "❌".to_string(),
            subtitle: "appcu".to_string(),
            message: msg,
            open_url: None,
            open_by_app: false,
        }
    }
    pub fn new_remote_get_failed(app_info: &AppInfo) -> Self {
        Notification {
            title: format!("❌{} 获取最新版本失败", app_info.name),
            subtitle: "appcu".to_string(),
            message: format!("{}", app_info.check_update_type),
            open_url: None,
            open_by_app: false,
        }
    }
    pub fn new_update_notification(
        name: String,
        local_version: String,
        remote_version: String,
        update_page_url: String,
        open_by_app: bool,
    ) -> Self {
        Notification {
            title: format!("🎉{} 有更新", name),
            subtitle: "点击通知下载最新版本安装包".to_string(),
            message: format!("{} -> {}", local_version, remote_version),
            open_url: Some(update_page_url),
            open_by_app,
        }
    }
    pub fn new_verbose_notification(
        app_info: &AppInfo,
        local_version: String,
        remote_info: &RemoteInfo,
        open_by_app: bool,
        open_url: Option<String>,
    ) -> Self {
        let open_url = if open_by_app && !app_info.is_mas_app() {
            format!("file://{}", open_url.unwrap_or_default())
        } else {
            remote_info.update_page_url.clone()
        };
        Notification {
            title: format!("{} 检查结束", app_info.name),
            subtitle: "点击通知下载最新版本安装包".to_string(),
            message: format!(
                "{} -> {}\n{}",
                local_version, remote_info.version, app_info.check_update_type
            ),
            open_url: Some(open_url),
            open_by_app,
        }
    }
    pub fn post(&self) {
        let mut terminal_notifier_path: String = TERMINAL_NOTIFIER_PATH.to_string();
        if terminal_notifier_path.is_empty() {
            terminal_notifier_path = "terminal-notifier".to_string()
        }
        let output = match &self.open_url {
            Some(open_url) => std::process::Command::new(terminal_notifier_path)
                .arg("-title")
                .arg(&self.title)
                .arg("-subtitle")
                .arg(&self.subtitle)
                .arg("-message")
                .arg(&self.message)
                .arg("-open")
                .arg(open_url)
                .output(),
            None => std::process::Command::new(terminal_notifier_path)
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
                std::io::ErrorKind::NotFound => println!("未能找到 `terminal-notifier` {:?}", self),
                _ => println!("{:?}", err),
            },
        }
    }
}
