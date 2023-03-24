use super::AppInfo;
use crate::{request::RemoteInfo, TERMINAL_NOTIFIER_PATH};

/// 通知信息结构体
#[derive(Debug)]
pub struct Notification {
    /// 通知标题
    pub title: String,
    /// 通知子标题
    pub subtitle: String,
    /// 通知信息
    pub message: String,
    /// 点击通知打开地址
    pub open_url: Option<String>,
    /// 是否点击打开应用
    pub open_by_app: bool,
}

impl Notification {
    /// 生成错误通知
    pub fn new_error_notification(msg: String) -> Self {
        Notification {
            title: "❌".to_string(),
            subtitle: "appcu".to_string(),
            message: msg,
            open_url: None,
            open_by_app: false,
        }
    }
    /// 生成获取版本信息错误的通知
    pub fn new_remote_get_failed(app_info: &AppInfo) -> Self {
        Notification {
            title: format!("❌{} 获取最新版本失败", app_info.name),
            subtitle: "appcu".to_string(),
            message: format!("{}", app_info.check_update_type),
            open_url: None,
            open_by_app: false,
        }
    }
    /// 生成更新的通知
    pub fn new_update_notification(
        name: String,
        local_version: String,
        remote_version: String,
        update_page_url: String,
        open_by_app: bool,
    ) -> Self {
        Notification {
            title: format!("🎉{} 有更新", name),
            subtitle: if open_by_app {
                "点击打开应用或MAS".to_string()
            } else {
                "点击通知下载最新版本安装包".to_string()
            },
            message: format!("{} -> {}", local_version, remote_version),
            open_url: Some(update_page_url),
            open_by_app,
        }
    }
    /// 生成详细信息通知
    pub fn new_verbose_notification(
        app_info: &AppInfo,
        local_version: String,
        remote_info: &RemoteInfo,
        open_by_app: bool,
        open_url: Option<String>,
    ) -> Self {
        Notification {
            title: format!("{} 检查结束", app_info.name),
            subtitle: if open_by_app { "点击打开应用或MAS".to_string() } else { "点击通知下载最新版本安装包".to_string() },
            message: format!(
                "{} -> {}\n{}",
                local_version, remote_info.version, app_info.check_update_type
            ),
            open_url,
            open_by_app,
        }
    }
    /// 通知发送
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
                .arg(open_url.replace(' ', "%20"))
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
