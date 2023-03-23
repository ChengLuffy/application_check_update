/// 测试获取系统版本
#[test]
fn get_arm_system_version_test() {
    let arm_system_version_str = appcu::local::plist::get_arm_system_version();
    assert!(!arm_system_version_str.is_empty())
}

#[test]
/// 测试获取应用信息
fn check_app_info_test() {
    let path = std::path::Path::new("tests/test_sources/");
    for item in std::fs::read_dir(path).unwrap() {
        assert!(item.is_ok());
        let app_path = item.unwrap().path();
        assert!(app_path.exists());
        let app_info = appcu::local::check_app_info(&app_path);
        if app_path.as_path().ends_with("unknown_app.app")
            || app_path.as_path().ends_with(".DS_Store")
        {
            assert!(app_info.is_none());
        } else {
            assert!(app_info.is_some());
            let app_info = app_info.unwrap();
            assert!(!app_info.bundle_id.is_empty());
            assert!(!app_info.short_version.is_empty());
            assert!(!app_info.version.is_empty());
        }
    }
}

#[test]
fn test_app_info() {
    let mas_path = std::path::Path::new("tests/test_sources/mas_app.app/").to_path_buf();
    let mas_app_info = appcu::local::check_app_info(&mas_path).unwrap();
    assert!(mas_app_info.is_mas_app());
    let sparkle_path = std::path::Path::new("tests/test_sources/sparkle_app.app/").to_path_buf();
    let sparkle_app_info = appcu::local::check_app_info(&sparkle_path).unwrap();
    assert!(sparkle_app_info.is_sparkle_app());
}
