//! 下载重试参数（纯逻辑，可测试）。

/// 下载重试次数（含首次）。
pub const MAX_DOWNLOAD_RETRIES: usize = 3;

/// 两次重试之间的等待秒数。
pub const RETRY_DELAY_SECS: u64 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_original() {
        assert_eq!(MAX_DOWNLOAD_RETRIES, 3);
        assert_eq!(RETRY_DELAY_SECS, 1);
    }
}
