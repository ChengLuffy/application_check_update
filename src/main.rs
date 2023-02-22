/// TODO: 提供 json 格式输出
/// TODO: 提供 terminal-notification 输出
fn main() {
    let args = std::env::args();
    if args.len() == 1 {
        appcu::check_all()
    } else {
        let mut args_vec: Vec<String> = args.into_iter().collect();
        let commmad = args_vec.remove(0);
        if commmad.ends_with("appcu") {
            let subcommand = args_vec.get(0).expect("识别命令出错");
            match subcommand.as_str() {
                "ignore" => {
                    args_vec.remove(0);
                    let temp = args_vec.get(0).map(|x| x.as_str());
                    match temp {
                        Some("-h") | Some("--help") => print_help_ignore(),
                        Some(&_) => appcu::local::config::ignore_some(args_vec),
                        None => println!("未能识别 ignore 参数，ignore 命令使用方式为 `appcu ignore /Applications/xx.app/ /Applications/yy.app/ ...`")
                    }
                }
                "alias" => {
                    args_vec.remove(0);
                    if args_vec.len() == 2 {
                        let bundle_id = &args_vec[0];
                        let alias_name = &args_vec[1];
                        appcu::local::config::alias(bundle_id, alias_name)
                    } else {
                        let first = args_vec.get(0).map(|x| x.as_str());
                        if first == Some("-h") || first == Some("--help") {
                            print_help_alias()
                        } else {
                            println!("未能识别 bundle_id 参数，alias 命令使用方式为 `appcu alias app.bundle.id alias_name`")
                        }
                    }
                }
                "generate_config" | "gc" => {
                    let temp = args_vec.get(1).map(|x| x.as_str());
                    match temp {
                        Some("-h") | Some("--help") => print_help_gc(),
                        Some(&_) => println!("错误的使用方式，For more information try '--help'"),
                        None => appcu::local::config::generate_config(),
                    }
                }
                "help" => {
                    let temp = args_vec.get(1).map(|x| x.as_str());
                    match temp {
                        Some("ignore") => print_help_ignore(),
                        Some("alias") => print_help_alias(),
                        Some("generate_config") | Some("gc") => print_help_gc(),
                        Some(&_) => println!("错误的使用方式"),
                        None => print_help(),
                    }
                }
                "--version" | "-V" => print_version(),
                "--help" | "-h" => print_help(),
                _ => appcu::check_some(args_vec),
            }
        }
    }
}

fn print_help() {
    println!("{}", HELP_STR)
}

fn print_help_ignore() {
    println!("{}", HELP_IGNORE_STR)
}

fn print_help_alias() {
    println!("{}", HELP_ALIAS_STR)
}

fn print_help_gc() {
    println!("{}", HELP_GC_STR)
}

fn print_version() {
    println!("appcu {}", env!("CARGO_PKG_VERSION"))
}

const HELP_STR: &str = "
macOS 应用检查更新

Usage:
  运行 `appcu` 对所有 `/Applications` 文件夹下的应用进行检查；
  运行 `appcu /Applications/xx.app /Applications/yy.app` 对特定应用进行检查；

Commands:
  ignore           忽略对应的应用
  alias            设置 HomeBrew 查询方式的应用别名
  generate_config  简写: `gc`，生成配置文件，详情请查看 `appcu help generate_config`
  help             Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
";

const HELP_IGNORE_STR: &str = "忽略对应的应用

Usage: appcu ignore /Applications/xx.app /Applications/yy.app

Options:
  -h, --help     Print help information";

const HELP_ALIAS_STR: &str = "设置 HomeBrew 查询方式的应用别名

Usage: appcu alias app.bundle.id alias_name

Options:
  -h, --help     Print help information";

const HELP_GC_STR: &str = "简写: `gc`，生成配置文件，详情请查看 `appcu help generate_config`

Usage: appcu generate_config

配置文件路径：`~/.config/appcu/config.yaml`
配置文件说明：
```
# 并行查询数量，默认 5，太多会导致错误
threads_num: 5

# 用于 App Store 备选区域查询，默认是当前登陆 Mac App Store 的账号区域，如果有一些应用是其他区域专属，可以在此添加
mas_area:
  # 例如我的主账号为美区账号，但是一些应用使用国区账号下载，所以将国区 `area_code` 添加在此处
  - cn

# HomeBrew 查询时，是将应用名称直接查寻，但是某些应用无法直接查到，可以在这里设置查询的别名
alias:
  # 例如：wps，获取当前安装的应用 bundle_id 为 `com.kingsoft.wpsoffice.mac`
  # 通过 `brew search wpsoffice` 选择 `wpsoffice-cn`，映射如下
  com.kingsoft.wpsoffice.mac: wpsoffice-cn

# 有些应用不用查询，或者无法查询（例如已经下架、未被收录在 HomeBrew 等），可以在这里设置忽略
ignore:
  # 例 safari 无法通过任何手段查询更新，获取 safari bundle_id 进行忽略
  # 也可以利用 `appcu ignore ...` 进行忽略
  - com.apple.Safari
```

Options:
  -h, --help  Print help information";
