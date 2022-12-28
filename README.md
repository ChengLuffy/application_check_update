# macOS 应用检查更新

```
macOS 应用检查更新

Usage: 
  运行 `appcu` 对所有 `/Applications` 文件夹下的应用进行检查；
  运行 `appcu /Applications/xx.app /Applications/yy.app` 对特定应用进行检查；

Commands:
  generate_config  生成配置文件
  ignore           忽略对应的应用
  help             Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```

### 配置文件
**配置文件读取路径：`～/.config/appcu/config.yaml`**
配置样例：
```
# 并行查询数量，默认 5，太多会导致错误
threads_num: 5

# 用于 App Store 备选区域查询，默认是当前登陆 Mac App Store 的账号区域，如果有一些应用是其他区域专属，可以在此添加
mas_area:
  - cn

# HomeBrew 查询时，是将应用名称直接查寻，但是某些应用无法直接查到，可以在这里设置查询的别名
alias:
  org.gimp.gimp-2.10: gimp
  com.kingsoft.wpsoffice.mac: wpsoffice-cn
  com.parallels.desktop.console: parallels
  org.jkiss.dbeaver.core.product: dbeaver-community
  com.jetbrains.intellij.ce: intellij-idea-ce
  com.bilibili.bilibiliPC: bilibili
  com.chamburr.Glance: glance-chamburr

# 有些应用不用查询，或者无法查询（例如已经下架、未被收录在 HomeBrew 等），可以在这里设置忽略
ignore:
  - org.pqrs.Karabiner-EventViewer
  - com.apple.Safari
  - org.gpgtools.gpgkeychain
```

## 为什么

### 为什么会有这个项目
macOS 安装应用方式多样，批量检查更新的有效方式很少，之前我一直在用 `MacUpdater` 但是他的 *2.0* 版本重新开始收费，*1.0* 版本不支持 macOS 13，所以考虑自己做一个。

### 为什么要保持自己的应用是最新的？
强迫症。

### 为什么采用 `Rust`？
其实这样的应用采用 `Swift` 应该更好，但是之前看到好多安利 `Rust` 的文章，想着自己试一试这门语言。

### 为什么写的这么烂？
开始我是想好好了解一下这门语言的，时值22年末尾，封控，阳性，家人阳性，远程工作，远程找工作，我后面对 `Rust` 这门语言语言的热情越来越低，而且 `Rust` 基础入门教程到处都是，谁都能搭建一个，而涉及到详细点的，很难找，中文的更少，特别是对于没有接触过 `C++` 的我来说，有点不友好。

`struct` 的 `Copy Clone` 这些 `trait` 的实现我都没找到相应的实现说明，也是我太菜了。

多线程的 `move` `channel` 也无法真正理解学会，查不到资料，也解决不了报错，自己太菜了。

多到不知道怎么处理的 `unwrap()`，只能用 `if let Some(...)` 防止程序中断，一层套一层，让人写起来也很烦躁。

神奇的 `crates.io`、`docs.rs`，完全看不懂，好多依赖库缺少关键的怎么去用的说明。

## License
应用采用 `MIT` 开源协议，详见 [LICENSE](LICENSE)