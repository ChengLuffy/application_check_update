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

### 配置文件相关
#### generate-config/gc
简写: `gc`，用于生成配置文件，详情请查看 `appcu help generate-config`

使用:
```
 appcu generate-config
```

配置文件路径：`~/.config/appcu/config.yaml`

配置文件字段说明：

- threads_num: 并行查询数量，默认 5，太多会导致错误
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

## 为什么

### 为什么会有这个项目
macOS 安装应用方式多样，批量检查更新的有效方式很少，之前我一直在用 `MacUpdater` 但是他的 *2.0* 版本重新开始收费，*1.0* 版本不支持 macOS 13，所以考虑自己做一个。

### 为什么采用 `Rust`？
其实这样的应用采用 `Swift` 应该更好，但是之前看到好多安利 `Rust` 的文章，想着自己试一试这门语言，但是由于自己学习方法的不成熟，总是用写 iOS 应用的经验套用在 `Rust` 上，所以写的并不好。

## License
应用采用 `MIT` 开源协议，详见 [LICENSE](LICENSE)