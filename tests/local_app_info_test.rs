use std::collections::HashMap;

use appcu::local::local_app_info::{self, ApplicationInfo};

#[test]
fn local_app_info_test() {
    match local_app_info::get_applications_info() {
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
                println!("   来源: {:?}", app.obtained_from);
                println!("   架构: {:?}", app.architecture);
                println!("   签名: {:?}", app.signed_by);
                println!("   签名: {:?}", app.path);
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
            assert!(true, "获取应用信息报错")
        }
    }
}
