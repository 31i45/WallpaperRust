//! 图源策略（纯逻辑，可测试）。
//!
//! 图源分两类：
//! - Direct：URL 模板直接指向图片（如 picsum）
//! - BingDaily：先请求 Bing JSON API 解析出 urlbase，再拼 UHD/HD 图片 URL
//!
//! 图源尝试顺序：Fisher-Yates 洗牌后逐个尝试，每个源至多一次。

/// 图源类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceKind {
    /// 直接图片 URL 模板（含 {w}/{h} 占位符）。
    Direct,
    /// Bing 每日壁纸：需先请求 JSON API 解析 urlbase。
    BingDaily,
}

/// 图源定义。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub name: &'static str,
    /// Direct: 图片 URL 模板；BingDaily: JSON API URL。
    pub url: &'static str,
    pub kind: SourceKind,
}

/// 内置图源列表。Bing 每日壁纸为主源（高质量 UHD），picsum 为兜底。
pub fn builtin_sources() -> Vec<Source> {
    vec![
        Source {
            name: "bing",
            url: "https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=8&mkt=zh-CN",
            kind: SourceKind::BingDaily,
        },
        Source {
            name: "picsum",
            url: "https://picsum.photos/{w}/{h}",
            kind: SourceKind::Direct,
        },
    ]
}

/// 将 URL 模板中的 {w}/{h} 替换为实际分辨率。
pub fn render_url(template: &str, width: u32, height: u32) -> String {
    template
        .replace("{w}", &width.to_string())
        .replace("{h}", &height.to_string())
}

/// 从 Bing JSON 响应中提取第一个 urlbase 字段（零依赖字符串解析）。
///
/// 响应形如：{"images":[{"urlbase":"/th?id=OHR.XXX_ZH-CN123",...}]}
pub fn extract_bing_urlbase(json: &str) -> Option<&str> {
    extract_all_bing_urlbases(json).into_iter().next()
}

/// 从 Bing JSON 响应中提取所有 urlbase 字段。
///
/// n=8 时返回 8 张图片的 urlbase，供随机选择增加多样性。
pub fn extract_all_bing_urlbases(json: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let marker = "\"urlbase\":\"";
    let mut start = 0;
    while let Some(pos) = json[start..].find(marker) {
        let abs_start = start + pos + marker.len();
        if let Some(end) = json[abs_start..].find('"') {
            result.push(&json[abs_start..abs_start + end]);
            start = abs_start + end;
        } else {
            break;
        }
    }
    result
}

/// 给定 Bing urlbase，返回候选图片 URL 列表（UHD 优先，HD 兜底）。
pub fn bing_image_urls(urlbase: &str) -> Vec<String> {
    vec![
        format!("https://www.bing.com{urlbase}_UHD.jpg"),
        format!("https://www.bing.com{urlbase}_1920x1080.jpg"),
    ]
}

/// 对图源列表做 Fisher-Yates 洗牌，返回一个新的有序列表。
pub fn shuffle_sources<R: FnMut() -> u64>(sources: &[Source], mut rng: R) -> Vec<Source> {
    let mut out: Vec<Source> = sources.to_vec();
    let n = out.len();
    for i in (1..n).rev() {
        let j = (rng() % (i as u64 + 1)) as usize;
        out.swap(i, j);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_url_replaces_placeholders() {
        assert_eq!(
            render_url("https://picsum.photos/{w}/{h}", 1920, 1080),
            "https://picsum.photos/1920/1080"
        );
    }

    #[test]
    fn render_url_no_placeholder_passthrough() {
        assert_eq!(
            render_url("https://example.com/x.jpg", 1, 1),
            "https://example.com/x.jpg"
        );
    }

    #[test]
    fn extract_bing_urlbase_from_real_response() {
        let json = r#"{"images":[{"urlbase":"/th?id=OHR.BabcockSP_ZH-CN5266848536","url":"/th?id=OHR.BabcockSP_ZH-CN5266848536_1920x1080.jpg"}]}"#;
        assert_eq!(
            extract_bing_urlbase(json),
            Some("/th?id=OHR.BabcockSP_ZH-CN5266848536")
        );
    }

    #[test]
    fn extract_all_bing_urlbases_from_multi_response() {
        let json = r#"{"images":[{"urlbase":"/th?id=OHR.AAA"},{"urlbase":"/th?id=OHR.BBB"},{"urlbase":"/th?id=OHR.CCC"}]}"#;
        let bases = extract_all_bing_urlbases(json);
        assert_eq!(bases.len(), 3);
        assert_eq!(bases[0], "/th?id=OHR.AAA");
        assert_eq!(bases[1], "/th?id=OHR.BBB");
        assert_eq!(bases[2], "/th?id=OHR.CCC");
    }

    #[test]
    fn extract_bing_urlbase_missing_returns_none() {
        assert_eq!(extract_bing_urlbase("{}"), None);
        assert_eq!(extract_bing_urlbase("no json here"), None);
        assert!(extract_all_bing_urlbases("{}").is_empty());
    }

    #[test]
    fn bing_image_urls_uhd_first_then_hd() {
        let urls = bing_image_urls("/th?id=OHR.Test_ZH-CN123");
        assert_eq!(urls.len(), 2);
        assert!(urls[0].ends_with("_UHD.jpg"));
        assert!(urls[1].ends_with("_1920x1080.jpg"));
        assert!(urls[0].starts_with("https://www.bing.com/"));
    }

    #[test]
    fn shuffle_preserves_all_elements() {
        let sources = builtin_sources();
        let mut seed = 0u64;
        let shuffled = shuffle_sources(&sources, || {
            seed += 7;
            seed
        });
        let mut a: Vec<_> = sources.iter().map(|s| s.name).collect();
        let mut b: Vec<_> = shuffled.iter().map(|s| s.name).collect();
        a.sort();
        b.sort();
        assert_eq!(a, b);
    }

    #[test]
    fn shuffle_each_source_appears_once() {
        let sources = builtin_sources();
        let shuffled = shuffle_sources(&sources, || 0);
        assert_eq!(shuffled.len(), sources.len());
        let mut names: Vec<_> = shuffled.iter().map(|s| s.name).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), sources.len());
    }

    #[test]
    fn builtin_sources_has_bing_and_picsum() {
        let names: Vec<_> = builtin_sources().iter().map(|s| s.name).collect();
        assert!(names.contains(&"bing"));
        assert!(names.contains(&"picsum"));
        assert!(!names.contains(&"imagecdn")); // 已死图源已移除
    }
}
