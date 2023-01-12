#[test]
fn mas_app_request_test() {
    let remote_info = appcu::request::area_check("com.apple.dt.Xcode", false);
    assert!(!remote_info.version.is_empty());
    assert!(!remote_info.update_page_url.is_empty());
    assert_ne!(remote_info.version, "-1", "MAS 应用信息获取失败");
}

#[test]
fn sparkle_feed_check_test() {
    let remote_info = appcu::request::sparkle_feed_check("https://www.iina.io/appcast.xml");
    assert!(!remote_info.version.is_empty());
    assert_ne!(remote_info.version, "-1");
    assert!(!remote_info.update_page_url.is_empty());
}

#[test]
fn homebrew_check_test() {
    let remote_info = appcu::request::homebrew_check("Android Studio", "com.google.android.studio");
    assert!(!remote_info.version.is_empty());
    assert_ne!(remote_info.version, "-1");
    assert!(!remote_info.update_page_url.is_empty());
}
