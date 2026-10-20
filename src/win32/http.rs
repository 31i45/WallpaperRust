//! 基于 WinHTTP 的图片下载（系统原生 TLS，自动跟随重定向）。
//!
//! 不依赖任何第三方 HTTP 库，仅使用系统自带的 winhttp.dll。

use super::*;
use crate::config::MAX_DOWNLOAD_BYTES;

/// 解析后的 URL 组件。
#[derive(Debug, PartialEq, Eq)]
pub struct ParsedUrl<'a> {
    pub host: &'a str,
    pub port: u16,
    pub path: &'a str,
    pub is_https: bool,
}

/// 解析 https?://host[:port]/path 形式的 URL（纯逻辑，可测试）。
pub fn parse_url(url: &str) -> Option<ParsedUrl<'_>> {
    let (is_https, rest) = if let Some(r) = url.strip_prefix("https://") {
        (true, r)
    } else {
        let r = url.strip_prefix("http://")?;
        (false, r)
    };

    let (authority, path) = match rest.find('/') {
        Some(idx) => (&rest[..idx], &rest[idx..]),
        None => (rest, "/"),
    };

    let (host, port) = if let Some(colon) = authority.rfind(':') {
        let port_str = &authority[colon + 1..];
        match port_str.parse::<u16>() {
            Ok(p) => (&authority[..colon], p),
            Err(_) => return None,
        }
    } else {
        (authority, if is_https { 443 } else { 80 })
    };

    if host.is_empty() {
        return None;
    }

    Some(ParsedUrl {
        host,
        port,
        path,
        is_https,
    })
}

/// 下载指定 URL 的内容，返回字节数据。
/// 超时：连接+读取共约 15 秒（WinHTTP 默认超时）。
pub fn download(url: &str) -> Option<Vec<u8>> {
    let parsed = parse_url(url)?;
    unsafe {
        let agent: Vec<u16> = concat!("WallpaperRust/", env!("CARGO_PKG_VERSION"))
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let session = WinHttpOpen(
            agent.as_ptr(),
            WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
            core::ptr::null(),
            core::ptr::null(),
            0,
        );
        if session.is_null() {
            return None;
        }

        let result = (|| -> Option<Vec<u8>> {
            let host: Vec<u16> = parsed.host.encode_utf16().chain(Some(0)).collect();
            let connect = WinHttpConnect(session, host.as_ptr(), parsed.port, 0);
            if connect.is_null() {
                return None;
            }
            let _connect_guard = HandleGuard(connect);

            let verb: Vec<u16> = "GET".encode_utf16().chain(Some(0)).collect();
            let obj: Vec<u16> = parsed.path.encode_utf16().chain(Some(0)).collect();
            let request = WinHttpOpenRequest(
                connect,
                verb.as_ptr(),
                obj.as_ptr(),
                core::ptr::null(),
                core::ptr::null(),
                core::ptr::null(),
                if parsed.is_https {
                    WINHTTP_FLAG_SECURE
                } else {
                    0
                },
            );
            if request.is_null() {
                return None;
            }
            let _req_guard = HandleGuard(request);

            // 允许跟随所有重定向（picsum 会 302 到实际图片地址）
            let policy = WINHTTP_OPTION_REDIRECT_POLICY_ALWAYS;
            WinHttpSetOption(
                request,
                WINHTTP_OPTION_REDIRECT_POLICY,
                &policy as *const _ as LPVOID,
                core::mem::size_of::<DWORD>() as DWORD,
            );

            if WinHttpSendRequest(
                request,
                WINHTTP_NO_ADDITIONAL_HEADERS,
                0,
                WINHTTP_NO_REQUEST_DATA,
                0,
                0,
                0,
            ) == 0
            {
                return None;
            }

            if WinHttpReceiveResponse(request, core::ptr::null_mut()) == 0 {
                return None;
            }

            // 检查 HTTP 状态码
            let mut status: DWORD = 0;
            let mut status_len = core::mem::size_of::<DWORD>() as DWORD;
            if WinHttpQueryHeaders(
                request,
                WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
                core::ptr::null(),
                &mut status as *mut _ as LPVOID,
                &mut status_len,
                core::ptr::null_mut(),
            ) == 0
                || !(200..300).contains(&status)
            {
                return None;
            }

            let mut data: Vec<u8> = Vec::new();
            loop {
                let mut available: DWORD = 0;
                if WinHttpQueryDataAvailable(request, &mut available) == 0 {
                    return None;
                }
                if available == 0 {
                    break;
                }
                if data.len() as u64 + available as u64 > MAX_DOWNLOAD_BYTES as u64 {
                    return None; // 即将超限，提前拒绝（防御恶意宣称超大 available 的 DoS）
                }
                let mut buf = vec![0u8; available as usize];
                let mut read: DWORD = 0;
                if WinHttpReadData(request, buf.as_mut_ptr() as LPVOID, available, &mut read) == 0 {
                    return None;
                }
                if read == 0 {
                    break;
                }
                data.extend_from_slice(&buf[..read as usize]);
                if data.len() > MAX_DOWNLOAD_BYTES {
                    return None; // 超大响应，可能是恶意构造
                }
            }

            Some(data)
        })();

        WinHttpCloseHandle(session);
        result
    }
}

/// 下载并以 UTF-8 字符串返回（用于 Bing JSON API 等文本响应）。
pub fn download_text(url: &str) -> Option<String> {
    let bytes = download(url)?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

struct HandleGuard(HANDLE);
impl Drop for HandleGuard {
    fn drop(&mut self) {
        unsafe {
            if !self.0.is_null() {
                WinHttpCloseHandle(self.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_https_default_port() {
        let u = parse_url("https://picsum.photos/1920/1080").unwrap();
        assert_eq!(u.host, "picsum.photos");
        assert_eq!(u.port, 443);
        assert_eq!(u.path, "/1920/1080");
        assert!(u.is_https);
    }

    #[test]
    fn parse_http_with_port() {
        let u = parse_url("http://example.com:8080/path/to/img.jpg?q=1").unwrap();
        assert_eq!(u.host, "example.com");
        assert_eq!(u.port, 8080);
        assert_eq!(u.path, "/path/to/img.jpg?q=1");
        assert!(!u.is_https);
    }

    #[test]
    fn parse_no_path_defaults_to_root() {
        let u = parse_url("https://example.com").unwrap();
        assert_eq!(u.path, "/");
    }

    #[test]
    fn parse_invalid_scheme_rejected() {
        assert!(parse_url("ftp://example.com/x").is_none());
        assert!(parse_url("not-a-url").is_none());
    }

    #[test]
    fn parse_empty_host_rejected() {
        assert!(parse_url("https:///path").is_none());
    }

    #[test]
    fn parse_invalid_port_rejected() {
        assert!(parse_url("https://example.com:abc/x").is_none());
        assert!(parse_url("https://example.com:99999/x").is_none());
    }

    // 注意：download 的集成测试不放在单元测试里，避免 CI 无网络时失败。
    // 端到端验证通过 e2e 脚本单独执行。
}
