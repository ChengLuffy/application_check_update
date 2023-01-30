use clap::Command;
use core::str;
use std::ffi::OsString;

/// TODO: 尝试使用 tui 输出 （是否要做，挺麻烦的）？
fn main() {
    let command = Command::new("appcu")
                            .name("appcu")
                            .about("macOS 应用检查更新")
                            .allow_external_subcommands(true)
                            .override_usage("\n  运行 `appcu` 对所有 `/Applications` 文件夹下的应用进行检查；\n  运行 `appcu /Applications/xx.app /Applications/yy.app` 对特定应用进行检查；")
                            .subcommand(Command::new("ignore")
                                                        .about("忽略对应的应用")
                                                        .override_usage("appcu ignore /Applications/xx.app /Applications/yy.app"))
                            .subcommand(Command::new("alias")
                                                        .about("设置 HomeBrew 查询方式的应用别名")
                                                        .override_usage("appcu alias app.bundle.id alias_name"))
                            .subcommand(Command::new("generate_config")
                                                        .alias("gc")
                                                        .about("简写: `gc`，生成配置文件，详情请查看 `appcu help generate_config`")
                                                        .override_usage("appcu generate_config\n\n配置文件路径：`~/.config/appcu/config.yaml`\n配置文件说明：\n```\n# 并行查询数量，默认 5，太多会导致错误\nthreads_num: 5\n\n# 用于 App Store 备选区域查询，默认是当前登陆 Mac App Store 的账号区域，如果有一些应用是其他区域专属，可以在此添加\nmas_area:\n  # 例如我的主账号为美区账号，但是一些应用使用国区账号下载，所以将国区 `area_code` 添加在此处\n  - cn\n\n# HomeBrew 查询时，是将应用名称直接查寻，但是某些应用无法直接查到，可以在这里设置查询的别名\nalias:\n  # 例如：wps，获取当前安装的应用 bundle_id 为 `com.kingsoft.wpsoffice.mac`\n  # 通过 `brew search wpsoffice` 选择 `wpsoffice-cn`，映射如下\n  com.kingsoft.wpsoffice.mac: wpsoffice-cn\n\n# 有些应用不用查询，或者无法查询（例如已经下架、未被收录在 HomeBrew 等），可以在这里设置忽略\nignore:\n  # 例 safari 无法通过任何手段查询更新，获取 safari bundle_id 进行忽略\n  # 也可以利用 `appcu ignore ...` 进行忽略\n  - com.apple.Safari\n```"))
                            .version("0.1.0");
    let args = command.get_matches();
    if let Some((external, ext_m)) = args.subcommand() {
        let mut ext_args: Vec<&str> = ext_m
            .get_many::<OsString>("")
            .unwrap_or_default()
            .map(|x| x.to_str().unwrap_or_default())
            .collect();
        let mut results = vec![external];
        results.append(&mut ext_args);
        if results.is_empty() {
            appcu::check_all()
        } else if results.len() == 1 && external == "generate_config" {
            appcu::local::config::generate_config()
        } else if external.starts_with("ignore") {
            if !external.starts_with("ignore ") {
                println!("未能识别 ignore 参数，ignore 命令使用方式为 `appcu ignore /Applications/xx.app/ /Applications/yy.app/ ...`")
            } else {
                let mut vec: Vec<&str> = external.split(' ').collect();
                vec.remove(0);
                if vec.is_empty() {
                    println!("未能识别 ignore 参数，ignore 命令使用方式为 `appcu ignore /Applications/xx.app/ /Applications/yy.app/ ...`")
                } else {
                    appcu::local::config::ignore_some(vec)
                }
            }
        } else if external.starts_with("alias") {
            if !external.starts_with("alias ") {
                println!("未能识别 bundle_id 参数，alias 命令使用方式为 `appcu alias app.bundle.id alias_name`")
            } else {
                let mut vec: Vec<&str> = external.split(' ').collect();
                vec.remove(0);
                if vec.len() == 2 {
                    let bundle_id = vec.first().unwrap();
                    if bundle_id.contains('.') {
                        let alias_name = vec.get(1).unwrap();
                        appcu::local::config::alias(bundle_id, alias_name)
                    } else {
                        println!("未能识别 bundle_id 参数，alias 命令使用方式为 `appcu alias app.bundle.id alias_name`")
                    }
                } else {
                    println!(
                        "未能识别命令，alias 命令使用方式为 `appcu alias app.bundle.id alias_name`"
                    )
                }
            }
        } else {
            appcu::check_some(results)
        }
    } else {
        appcu::check_all()
    }
}
