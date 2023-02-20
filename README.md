# macOS 应用检查更新

```
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
```

### 安装

目前应用未经受测试，也有一些无法解决的问题，比如：“通过 Homebrew api 查询应用更新时，有些应用无法正确的比对版本”，如果您想安装使用，可以下载源码后进行编译使用。

```
// 克隆仓库
git clone https://github.com/ChengLuffy/application_check_update.git appcu

// 进入文件夹
cd appcu

// 编译发行版本，如果没有安装 Rust 环境，请先搜索安装
cargo build --release

// 拷贝到任何一个 $PATH 包含的文件夹下
cp target/release/appcu /usr/local/bin/

// 查看是否运行正常
appcu -h
```


### 使用说明
#### `generate_config`
```
简写: `gc`，生成配置文件，详情请查看 `appcu help generate_config`

Usage: appcu generate_config

配置文件路径：`~/.config/appcu/config.yaml`
配置文件说明：

# 并行查询数量，默认 5，太多会导致错误
threads_num: 5

# 用于 App Store 备选区域查询，默认是当前登陆 Mac App Store 的账号区域
# 如果有一些应用是其他区域专属，可以在此添加
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

Options:
  -h, --help  Print help information
```

#### `ignore`
```
忽略对应的应用

Usage: appcu ignore /Applications/xx.app /Applications/yy.app

Options:
  -h, --help  Print help information
```

#### `alias`
```
设置 HomeBrew 查询方式的应用别名

Usage: appcu alias app.bundle.id alias_name

Options:
  -h, --help  Print help information
```

## 为什么

### 为什么会有这个项目
macOS 安装应用方式多样，批量检查更新的有效方式很少，之前我一直在用 `MacUpdater` 但是他的 *2.0* 版本重新开始收费，*1.0* 版本不支持 macOS 13，所以考虑自己做一个。

### 为什么要保持自己的应用是最新的？
强迫症。

### 为什么采用 `Rust`？
其实这样的应用采用 `Swift` 应该更好，但是之前看到好多安利 `Rust` 的文章，想着自己试一试这门语言，但是由于自己学习方法的不成熟，总是用写 iOS 应用的经验套用在 `Rust` 上，所以写的并不好。

## License
应用采用 `MIT` 开源协议，详见 [LICENSE](LICENSE)