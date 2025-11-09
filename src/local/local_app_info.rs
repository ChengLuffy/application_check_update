use std::collections::HashMap;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApplicationInfo {
    #[serde(rename = "_name")]
    pub name: String,
    #[serde(rename = "obtained_from")]
    pub obtained_from: String,
    pub version: Option<String>,
    #[serde(rename = "lastModified")]
    pub last_modified: String,
    pub kind: Option<String>,
    #[serde(rename = "arch_kind")]
    pub architecture: String,
    #[serde(rename = "signed_by")]
    pub signed_by: Option<Vec<String>>,
    pub path: Option<String>,
}

pub fn get_applications_info() -> Result<HashMap<String, ApplicationInfo>, Box<dyn std::error::Error>> {
    let output = Command::new("system_profiler")
        .args(&["SPApplicationsDataType", "-json"])
        .output()?;

    if !output.status.success() {
        return Err(format!("命令执行失败: {}", String::from_utf8_lossy(&output.stderr)).into());
    }

    let json_str = String::from_utf8(output.stdout)?;

    // 直接解析为 HashMap
    let parsed: HashMap<String, Vec<ApplicationInfo>> = serde_json::from_str(&json_str)?;
    
    if let Some(apps) = parsed.get("SPApplicationsDataType") {
        let applications: HashMap<String, ApplicationInfo> = apps
            .iter()
            .map(|app| (app.name.clone(), app.clone()))
            .collect();
        Ok(applications)
    } else {
        Err("未找到应用数据".into())
    }
}

// 扩展功能：应用过滤
pub struct ApplicationFilter<'a> {
    pub name_contains: Option<&'a str>,
    pub obtained_from: Option<&'a str>,
    pub architecture: Option<&'a str>,
}

impl ApplicationInfo {
    pub fn matches_filter(&self, filter: &ApplicationFilter) -> bool {
        if let Some(name) = filter.name_contains {
            if !self.name.to_lowercase().contains(&name.to_lowercase()) {
                return false;
            }
        }
        
        if let Some(source) = filter.obtained_from {
            if self.obtained_from != source {
                return false;
            }
        }
        
        if let Some(arch) = filter.architecture {
            if self.architecture != arch {
                return false;
            }
        }
        
        true
    }
}

// 使用过滤功能
pub fn filter_applications() -> Result<(), Box<dyn std::error::Error>> {
    let applications = get_applications_info()?;
    
    let filter = ApplicationFilter {
        name_contains: Some("chrome"),
        obtained_from: Some("Identified Developer"),
        architecture: None,
    };
    
    println!("过滤结果:");
    for app in applications.values().filter(|a| a.matches_filter(&filter)) {
        println!("  - {} (来源: {})", app.name, app.obtained_from);
    }
    
    Ok(())
}

#[allow(dead_code)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    match get_applications_info() {
        Ok(applications) => {
            println!("找到 {} 个应用", applications.len());
            
            // 按来源分类统计
            let mut by_source: HashMap<String, Vec<&ApplicationInfo>> = HashMap::new();
            for app in applications.values() {
                by_source.entry(app.obtained_from.clone())
                    .or_insert_with(Vec::new)
                    .push(app);
            }
            
            // 打印统计信息
            println!("\n应用来源统计:");
            for (source, apps) in &by_source {
                println!("  {}: {} 个应用", source, apps.len());
            }
            
            // 打印前10个应用的信息
            println!("\n前10个应用详情:");
            for (i, app) in applications.values().take(10).enumerate() {
                println!("{}. {}", i + 1, app.name);
                println!("   版本: {:?}", app.version);
                println!("   来源: {}", app.obtained_from);
                println!("   架构: {}", app.architecture);
                println!("   签名: {:?}", app.signed_by);
                println!();
            }
            
            // 查找特定来源的应用
            println!("来自 App Store 的应用:");
            for app in applications.values().filter(|a| a.obtained_from == "Apple") {
                println!("  - {} (v{:?})", app.name, app.version);
            }
        }
        Err(e) => {
            eprintln!("错误: {}", e);
        }
    }
    
    Ok(())
}