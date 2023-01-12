use std::error::Error;

#[test]
fn remote_default_config_test() {
    if let Ok(content) = get_default_config() {
        let _: appcu::local::config::Config = serde_yaml::from_str(&content).unwrap();
    } else {
        panic!("获取默认配置错误")
    }
}

#[tokio::main]
async fn get_default_config() -> Result<String, Box<dyn Error>> {
    let content = reqwest::get("https://raw.githubusercontent.com/ChengLuffy/application_check_update/master/default_config.yaml").await?.text().await?;
    Ok(content)
}
