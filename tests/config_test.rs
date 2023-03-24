use std::error::Error;

#[test]
fn remote_default_config_test() {
    if let Ok(content) = get_default_config() {
        let _: appcu::local::config::Config = serde_yaml::from_str(&content).unwrap();
    } else {
        panic!("获取默认配置错误")
    }
    let config = appcu::local::config::Config::default();
    assert_eq!(config.threads_num, 5);
    assert!(config.terminal_notifier_path.is_empty());
    assert!(config.mas_area.is_empty());
    assert!(config.alias.is_empty());
}

#[tokio::main]
async fn get_default_config() -> Result<String, Box<dyn Error>> {
    let content = reqwest::get("https://raw.githubusercontent.com/ChengLuffy/application_check_update/master/default_config.yaml").await?.text().await?;
    Ok(content)
}
