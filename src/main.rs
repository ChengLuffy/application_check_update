use clap::{arg, Parser, Subcommand};
use std::ffi::OsString;

/// TODO: 提供 json 格式输出

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "appcu", version)]
#[command(author = "chengluffy. <chengluffy@gmail.com>")]
#[command(about = "macOS 应用检查更新\n运行 `appcu` 对所有 `/Applications/` 文件夹下的应用进行检查\n运行 `appcu /Applications/xx.app /Applications/yy.app` 对特定应用进行检查", long_about = None)]
#[command(subcommand_required = false)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    /// 是否以系统通知的形式输出检查更新结果，需要安装 `terminal-notifier`
    #[arg(short = 'n')]
    notification: bool,
    /// 详细输出，仅支持检查更新的输出
    #[arg(short = 'v')]
    verbose: bool,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// 忽略对应的应用
    #[command(arg_required_else_help = true)]
    Ignore {
        /// 应用路径
        app_paths: Vec<OsString>,
    },
    /// 设置 HomeBrew 查询方式的应用别名
    #[command(arg_required_else_help = true)]
    Alias {
        /// 应用地址
        app_path: OsString,
        /// 查询别名
        alias_name: OsString,
    },
    /// 别名: `gc`，生成配置文件
    #[command(alias = "gc")]
    GenerateConfig,
    #[command(external_subcommand)]
    External(Vec<OsString>),
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Ignore { app_paths }) => appcu::local::config::ignore_some(app_paths),
        Some(Commands::Alias {
            app_path,
            alias_name,
        }) => appcu::local::config::alias(app_path, alias_name),
        Some(Commands::GenerateConfig) => appcu::local::config::generate_config(),
        Some(Commands::External(args)) => {
            let notification = cli.notification;
            let verbose = cli.verbose;
            let paths: Vec<String> = args
                .into_iter()
                .map(|x| x.into_string().unwrap_or_default())
                .collect();
            let filter_result: Vec<String> = paths
                .iter()
                .filter(|x| !x.contains(".app"))
                .cloned()
                .collect();
            if !filter_result.is_empty() {
                println!(
                    "识别到不支持的路径：{}\nOPTIONS 需要放到 PATHS 前面",
                    filter_result.join(", ")
                );
            } else {
                let check_operation = appcu::CheckOperation {
                    notification,
                    verbose,
                };
                check_operation.check_some(paths)
            }
        }
        None => {
            let notification = cli.notification;
            let verbose = cli.verbose;
            let check_operation = appcu::CheckOperation {
                notification,
                verbose,
            };
            check_operation.check_all()
        }
    }
}
