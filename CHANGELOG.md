# CHANGELOG

## Unreleased
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
