# CHANGELOG

## Unreleased

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
  - 修复 `generate_config` 即使输入 `y` 也无法正常运行的问题
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
