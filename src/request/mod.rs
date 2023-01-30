use super::{version_cmp, ALIAS, ARM_SYSTEM_NAME, MASAREAS};
use rss::Channel;
use skyscraper::html;

#[derive(Debug)]
pub struct RemoteInfo {
    pub version: String,
    pub update_page_url: String,
}

////////////////////////////////////////////////////////////////////////////////
// 网络请求
////////////////////////////////////////////////////////////////////////////////

/// 查询 HomeBrew-Casks 内的版本信息
///
/// 读取 `Homebrew/homebrew-cask` 仓库 `Casks` 文件夹内的响应应用文件
#[tokio::main]
pub async fn homebrew_check(app_name: &str, bundle_id: &str) -> RemoteInfo {
    let dealed_app_name = app_name.to_lowercase().replace(' ', "-");
    let alias_keys = ALIAS.keys();
    let file_name = if !alias_keys.into_iter().any(|x| x == &bundle_id.to_string()) {
        &dealed_app_name
    } else {
        &ALIAS[bundle_id]
    };
    if let Ok(resp) = reqwest::get(format!(
        "https://formulae.brew.sh/api/cask/{file_name}.json"
    ))
    .await
    {
        if let Ok(text) = resp.text().await {
            if let Ok(json_value) = serde_json::from_str(&text) {
                let json_value: serde_json::Value = json_value;
                let version_arr: Vec<&str> = json_value
                    .get("version")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .split(',')
                    .collect();
                let mut version: &str = version_arr.first().unwrap_or(&"");
                let arch_str = std::env::consts::ARCH;
                let mut url = json_value
                    .get("url")
                    .unwrap()
                    .as_str()
                    .unwrap_or_default()
                    .to_string();
                // FIXME: 由于自己受伤只有一个 M1 Pro mac 无法确定下面的判断是否正确
                if ARM_SYSTEM_NAME.len() > 0 && (arch_str == "aarch64" || arch_str == "arm") {
                    if let Some(variations) = json_value.get("variations") {
                        if let Some(arm64_system_name) = variations.get(ARM_SYSTEM_NAME.as_str()) {
                            if let Some(url_value) = arm64_system_name.get("url") {
                                let url_temp = url_value.as_str().unwrap_or_default().to_string();
                                url = url_temp;
                            }
                            // 个别应用，例如 dingtalk 7.0.x 时，M 系列版本和 intel 版本不是一个版本号
                            if let Some(version_value) = arm64_system_name.get("version") {
                                let version_temp = version_value.as_str().unwrap_or_default();
                                version = version_temp;
                            }
                        }
                    }
                }
                return RemoteInfo {
                    version: version.to_string(),
                    update_page_url: url,
                };
            } else {
                return RemoteInfo {
                    version: "".to_string(),
                    update_page_url: String::new(),
                };
            }
        }
    }
    RemoteInfo {
        version: "-1".to_string(),
        update_page_url: String::new(),
    }
}

#[tokio::main]
pub async fn sparkle_feed_check(feed_url: &str) -> RemoteInfo {
    if let Ok(content) = reqwest::get(feed_url).await {
        if let Ok(bytes_content) = content.bytes().await {
            if let Ok(channel) = Channel::read_from(&bytes_content[..]) {
                let mut items: Vec<rss::Item> = channel.items().into();
                // FIXME: 有些应用例如 playcover 2.0.1 版本，应用内 Info.plist 的版本却是 2.0.0 ，但是 shortVersion 又不是所有应用都有的
                // FIXME: xml 格式也不统一，有些把版本信息放在 enclosure 内，有些是直接是标题，有些是 item 内
                items.sort_by(|a, b| {
                    if let Some(a_enclosure) = a.enclosure() {
                        if let Some(b_enclosure) = b.enclosure() {
                            let mut a_version = a_enclosure.version.as_str();
                            if !a_version.contains('.') {
                                a_version = a_enclosure.short_version.as_str();
                            }
                            if a_version.is_empty() {
                                a_version = a.title().unwrap_or_default();
                            }
                            let mut b_version = b_enclosure.version.as_str();
                            if !b_version.contains('.') {
                                b_version = b_enclosure.short_version.as_str();
                            }
                            if b_version.is_empty() {
                                b_version = b.title().unwrap_or_default();
                            }
                            return version_cmp::cmp_version(a_version, b_version, true);
                        }
                    }
                    std::cmp::Ordering::Equal
                });
                if let Some(item) = items.last() {
                    let mut version = item.version().unwrap_or_default();
                    if version.is_empty() {
                        version = item.short_version().unwrap_or_default();
                    }
                    if version.is_empty() {
                        if let Some(enclosure) = item.enclosure() {
                            let mut version = enclosure.version.as_str();
                            if !version.contains('.') {
                                version = &enclosure.short_version;
                            }
                            if version.is_empty() {
                                version = item.title().unwrap_or_default();
                            }
                            let result = RemoteInfo {
                                version: version.to_string(),
                                update_page_url: enclosure.url.to_string(),
                            };
                            return result;
                        }
                    }
                    let result = RemoteInfo {
                        version: version.to_string(),
                        update_page_url: item.enclosure().unwrap().url.to_string(),
                    };
                    return result;
                }
            }
        }
    }
    RemoteInfo {
        version: "-1".to_string(),
        update_page_url: String::new(),
    }
}

/// MAS 应用和 iOS 应用可能存在区域内未上架的问题，采取先检测 cn 后检测 us 的方式
pub fn area_check(bundle_id: &str, is_ios_app: bool) -> RemoteInfo {
    let mut mas_areas = MASAREAS.to_vec();
    mas_areas.insert(0, "".to_string());
    for area_code in mas_areas {
        let remote_info_opt = mas_app_check(&area_code, bundle_id, is_ios_app);
        if let Some(remote_info) = remote_info_opt {
            return remote_info;
        }
    }
    RemoteInfo {
        version: String::new(),
        update_page_url: "".to_string(),
    }
}

/// 通过 itunes api 查询应用信息
#[tokio::main]
async fn mas_app_check(area_code: &str, bundle_id: &str, is_ios_app: bool) -> Option<RemoteInfo> {
    if let Ok(resp) = reqwest::get(format!(
        "https://itunes.apple.com/{area_code}/lookup?bundleId={bundle_id}"
    ))
    .await
    {
        if let Ok(text) = resp.text().await {
            let json_value: serde_json::Value = serde_json::from_str(&text).unwrap();
            let result_count = json_value
                .get("resultCount")
                .unwrap()
                .as_u64()
                .unwrap_or_default();
            if result_count != 0 {
                let results = json_value.get("results").unwrap();
                let item = results.get(0).unwrap();
                let update_page_url = item
                    .get("trackViewUrl")
                    .unwrap()
                    .to_string()
                    .replace('\"', "");
                let mut version = item.get("version").unwrap().to_string().replace('\"', "");
                // iOS 和 iPadOS 的应用不需要走这个流程
                if !is_ios_app {
                    // FIXME: 某些 iOS 和 macOS 应用使用一样的 bundleid 现在的查询方法只会返回 iOS 的结果，例如：ServerCat PasteNow，暂时的解决方案：抓取网页数据，匹配 <p class="l-column small-6 medium-12 whats-new__latest__version">Version/版本 x.x.x</p>
                    // FIXME: 上述方案会偶发性查不到，原因是通过 trackViewUrl 获取的 html 文本可能是没查到信息前的 loading 文本，所以 loop 一下
                    // FIXME: 还有一种情况，例如 QQ 6.9.0 通过 iTunes api cn 可以查到 6.9.0 版本，但是 us 还是 6.8.9，所以统一改成再用应用主页查一遍
                    let client = reqwest::Client::new();
                    loop {
                        if let Ok(resp) = client.get(&update_page_url).header("USER_AGENT", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.2 Safari/605.1.15").send().await {
                          if let Ok(text) = resp.text().await {
                              if let Ok(document) = html::parse(&text) {
                                  let xpath = skyscraper::xpath::parse::parse("//p[@class='l-column small-6 medium-12 whats-new__latest__version']").unwrap();
                                  if let Ok(nodes) = xpath.apply(&document) {
                                      if let Some(doc_node) = nodes.get(0) {
                                          if let Some(text) = doc_node.get_text(&document) {
                                              if let Some(last) = text.split(' ').last() {
                                                  version = last.to_string();
                                                  break;
                                              }
                                          }
                                      }
                                  }
                              }
                          }
                      }
                    }
                }
                return Some(RemoteInfo {
                    version,
                    update_page_url,
                });
            } else {
                return None;
            }
        }
    }
    Some(RemoteInfo {
        version: "-1".to_string(),
        update_page_url: "".to_string(),
    })
}
