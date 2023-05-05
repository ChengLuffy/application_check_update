# CHANGELOG

## Unreleased

## [0.1.8] - 2023-05-05
### Fixes
  - 修复 MAS 查询方式在请求失败的情况下应用崩溃的问题
  - 修复查询失败没有阻断继续输出的情况
  - 修复已经配置忽略的应用依然有远程信息查询输出

## [0.1.7] - 2023-03-27
### Added
  - 添加 `-o` 选项，当使用此选项时，输出结果打印应用路径或 MAS 链接，点击通知时打开应用或者 MAS 主页

## [0.1.6] - 2023-03-16
### Fixes
  - 由于 crontab 环境原因，移除通过 `type` 或 `which` 检测 `terminal-notifier` 安装路径的方式

### Added
  - 配置文件新增 `terminal_notifier_path` 字段，供调用 `terminal-notifier` 时使用
  - `generate-config/gc` 自动生成默认配置，不再从网络获取默认配置
  - 生成默认配置时尝试读取 `terminal-notifier` 安装路径

## [0.1.5] - 2023-03-15
### Fixes
  - 修复无法正常运行在 crontab 的问题
  - 修复 rss 依赖警告的问题

## [0.1.4] - 2023-03-10
### Fixes
  - 修复一些应用 info.plist 内不存在 CFBundleShortVersionString

## [0.1.3] - 2023-03-10
### Fixes
  - 修复某些应用 Wrapper/iTunesMetadata.plist 内不包括版本号的问题

### Added
  - 使用 clap 管理解析命令
  - 增加 -n，使用 `terminal-notifier` 发送检查更新结果，**点击通知后直接下载或跳转到相应页面**
  - 增加 -v，输出所有详细信息，包括应用信息、应用匹配的检查更新方式、应用最新版本地址、应用版本号，**包括未发现新版本的应用信息**

## [0.1.2] - 2023-02-22
### Fixes
  - 修复 `--version` 输出错误的版本信息
  - 修复 `generate-config` 即使输入 `y` 也无法正常运行的问题
  - 修复 `ignore` `alias` 等命令无法按预期运行的问题
### Added
  - 添加安装脚本

## [0.1.1] - 2023-02-17
### Fixes
  - 限制每个应用最多查五次，解决死循环的情况出现；
### CD
  - 同时生成上传 arm64、x86_64 编译包

## [0.1.0] - 2023-02-17
初始版本
