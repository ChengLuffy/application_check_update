# macOS 应用检查更新

一个用于检查 macOS 应用更新的 CLI 工具

## 使用说明

### 开始检查应用更新

对所有 **/Applications** 路径下应用检查更新：
```
appcu
```

对特定路径应用检查更新：
```
appcu /Applications/xx.app /Applications/yy.app
```

### 参数说明
#### generate-config/gc
简写: `gc`，用于生成配置文件，详情请查看 `appcu help generate-config`

使用:
```
 appcu generate-config
```

配置文件路径：`~/.config/appcu/config.yaml`

配置文件字段说明：

- threads_num: 并行查询数量，默认 5，太多会导致错误
- terminal_notifier_path: terminal-notifier 的安装路径，使用 `--notification/-n` 前需要预先配置该字段，可以通过 `which terminal-notifier` 查询
- mas_area: 用于 App Store 备选区域查询，默认是当前登陆 Mac App Store 的账号区域，如果有一些应用是其他区域专属，可以在此添加作为备选查询区域
- alias: HomeBrew 查询时，是将应用名称直接查寻，但是某些应用无法直接查到，可以在这里设置查询的别名。例如：wps，通过 `brew search wpsoffice` 选择 `wpsoffice-cn`，使用 `appcu alias /Applications/wpsoffice.app/ wpsoffice-cn` 进行映射
- ignore: 有些应用不用查询，或者无法查询（例如已经下架、未被收录在 HomeBrew 等），可以在这里设置忽略，例 Safari Technology Preview 无法通过任何手段查询更新，使用 `appcu ignore /Applications/Safari\ Technology\ Preview.app/` 进行忽略

#### ignore
忽略对应的应用

使用:
```
appcu ignore /Applications/xx.app /Applications/yy.app
```

#### alias
设置 HomeBrew 查询方式的应用别名

使用:
```
appcu alias /Applications/xx.app alias_name
```

### -n
使用 `terminal-notifier` 发送检查更新结果，**点击通知后直接下载或跳转到相应页面**

需要预先安装 [`terminal-notifier`](https://github.com/julienXX/terminal-notifier)

结合 `crontab` 可以实现定时检查更新，例如每天早上10点运行
```
0 10 * * * appcu -n
```

### -v
输出所有详细信息，包括应用信息、应用匹配的检查更新方式、应用最新版本地址、应用版本号，**包括未发现新版本的应用信息**。

### --version/-V
输出版本信息

## 安装

### 使用 Homebrew

`brew install chengluffy/appcu/appcu`

或者 `brew tap chengluffy/appcu` 然后 `brew install appcu`.

### 使用脚本

脚本内容: [appcu-install.sh](appcu-install.sh)
```
sudo bash -c "$(curl -fsSL https://raw.githubusercontent.com/chengluffy/application_check_update/master/appcu-install.sh)"
```

### 自行编译
需要 `rust` 环境
- 克隆仓库: `git clone https://github.com/ChengLuffy/application_check_update.git appcu && cd appcu`
- 编译发行版本: `cargo build --release`
- 拷贝到任何一个 $PATH 包含的文件夹下，例如: `cp target/release/appcu /usr/local/bin/`
- 查看是否运行正常: `appcu -h`

## 查询方式说明
appcu 不提供应用的版本信息数据库，appcu 仅通过 iTunes API、应用官方的 Sparkle 链接、Homebrew-Cask API 进行查询，这意味着 appcu 的开发者无法获取到您的电脑上安装了哪些应用，同时 appcu 也不会收集您的任何信息

注意：下面介绍的顺序即为 appcu 判断应用需要采用那种方式检查更新的顺序
### MAS 安装的应用
1. Apple Silicon 支持的 macOS 上，通过 Mac App Store 安装的 iPad 应用

可以通过验证 `xx.app/Wrapper/iTunesMetadata.plist` 文件存在，确认使用该方式

需要注意的是某些 **企业证书** 签发的应用虽然存在该文件，但是无法检查更新。

大部分应用可以通过 `iTunesMetadata.plist` 读取到 bundle_id 值，但有些应用不可以，这时需要读取 `Info.plist` 文件获取

此类应用检查更新方式为：通过 iTunes API 查询

```
https://itunes.apple.com/{area_code}/lookup?bundleId={bundle_id}
```

2. 其他通过 Mac App Store 安装的应用

可以通过验证 `xx.app/Contents/_MASReceipt` 文件夹存在，确认使用该方式

此类应用由于某些开发者在 iOS iPadOS macOS 的应用采用同样的 bundle_id，所以仅通过 iTunes API 无法确认 macOS 的版本号，需要基于 iTunes API 的返回结果中的 trackViewUrl 字段，获取 web 详情页面内容，通过 xpath 获取 `//p[@class='l-column small-6 medium-12 whats-new__latest__version']` 响应内容，解析后即为 macOS 应用的版本号。

### 通过 Sparkle 分发版本的应用
可以通过验证 `Info.plist` 内存在 `SUFeedURL` 字段确认使用该方式

请求 `SUFeedURL`，其内容类似 rss，解析可得应用版本号

需要注意的是，有些应用虽然可以通过其他方式确认是通过 Sparkle 分发版本的，但是 `Info.plist` 内不存在 `SUFeedURL` 字段，这时无法通过该方式获得应用版本号

关于版本号，有些应用是通过 `CFBundleShortVersionString` 进行比较的，有些应用是通过 `CFBundleVersion` 进行比较的，这取决于 `SUFeedURL` 提供的是什么值

### 其他应用
通过 Homebrew-Cask API 进行查询

```
https://formulae.brew.sh/api/cask/{app_name}.json
```

app_name 其实就是应用的名称，需要将大写字母转换为小写字母，同时用 `-` 替换空格

需要注意的是，很多应用在 Hombrew-Cask 的名称和用户机器上的名称不一致，所以 appcu 提供了设置应用别名的方式

有些应用在 Apple Silicon 和 Intel 平台发布的是两个安装包，可以通过解析 Homebrew-Cask API 结果中的 `variations` 字段内容确认应该下载哪个版本

### 通过上述方法查询不到版本信息的应用
appcu 提供了忽略检查更新的方式

## 为什么

### 为什么会有这个项目
macOS 安装应用方式多样，批量检查更新的有效方式很少，之前我一直在用 `MacUpdater` 但是他的 *2.0* 版本重新开始收费，*1.0* 版本不支持 macOS 13，所以考虑自己做一个。

### 为什么采用 `Rust`？
其实这样的应用采用 `Swift` 应该更好，但是之前看到好多安利 `Rust` 的文章，想着自己试一试这门语言，但是由于自己学习方法的不成熟，总是用写 iOS 应用的经验套用在 `Rust` 上，所以写的并不好。

## License
应用采用 `MIT` 开源协议，详见 [LICENSE](LICENSE)