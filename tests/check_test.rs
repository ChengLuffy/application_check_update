#[test]
#[ignore]
fn check_update_test() {
    let wechat_path = std::path::Path::new("/Applications/WeChat.app/");
    let wechat_app_info = appcu::local::check_app_info(wechat_path).unwrap();
    assert!(matches!(
        wechat_app_info.check_update_type,
        appcu::local::CheckUpType::Mas {
            bundle_id: _,
            is_ios_app: _
        }
    ));

    let weibo_path = std::path::Path::new("/Applications/Weibo.app/");
    let weibo_app_info = appcu::local::check_app_info(weibo_path).unwrap();
    assert!(matches!(
        weibo_app_info.check_update_type,
        appcu::local::CheckUpType::Mas {
            bundle_id: _,
            is_ios_app: true
        }
    ));

    let code_path = std::path::Path::new("/Applications/Visual Studio Code.app/");
    let code_app_info = appcu::local::check_app_info(code_path).unwrap();
    assert!(matches!(
        code_app_info.check_update_type,
        appcu::local::CheckUpType::HomeBrew {
            app_name: _,
            bundle_id: _
        }
    ));

    let iina_path = std::path::Path::new("/Applications/IINA.app/");
    let iina_app_info = appcu::local::check_app_info(iina_path).unwrap();
    assert!(matches!(
        iina_app_info.check_update_type,
        appcu::local::CheckUpType::Sparkle(_)
    ));
}
