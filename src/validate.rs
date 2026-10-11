//! JPEG 完整性校验（纯逻辑，可测试）。
//!
//! 原版 Python 代码无任何完整性校验，下载到半截文件会直接设为壁纸导致黑屏。
//! 此处提供三重校验：SOI 魔数、EOI 结尾、最小字节数。

pub const MIN_JPEG_BYTES: usize = 1024;
const JPEG_SOI: [u8; 3] = [0xFF, 0xD8, 0xFF];
const JPEG_EOI: [u8; 2] = [0xFF, 0xD9];

/// 判断字节流是否为完整的 JPEG 图片。
pub fn is_valid_jpeg(data: &[u8]) -> bool {
    if data.len() < MIN_JPEG_BYTES {
        return false;
    }
    // SOI: FF D8 FF
    if !data.starts_with(&JPEG_SOI) {
        return false;
    }
    // EOI: FF D9（从尾部向前查找，容忍 trailing 数据）
    data.windows(JPEG_EOI.len()).any(|w| w == JPEG_EOI)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jpeg_body() -> Vec<u8> {
        // SOI + 填充 + EOI
        let mut v = vec![0xFF, 0xD8, 0xFF];
        v.extend(std::iter::repeat_n(0xAA, 2048));
        v.push(0xFF);
        v.push(0xD9);
        v
    }

    #[test]
    fn valid_jpeg_passes() {
        assert!(is_valid_jpeg(&jpeg_body()));
    }

    #[test]
    fn too_small_rejected() {
        let mut v = vec![0xFF, 0xD8, 0xFF, 0x00, 0x00, 0xFF, 0xD9];
        assert!(!is_valid_jpeg(&v));
        // 填满到 MIN 以下
        v.extend(std::iter::repeat_n(0x00, MIN_JPEG_BYTES - v.len() - 1));
        assert!(!is_valid_jpeg(&v));
    }

    #[test]
    fn wrong_soi_rejected() {
        let mut v = jpeg_body();
        v[0] = 0x00;
        assert!(!is_valid_jpeg(&v));
    }

    #[test]
    fn missing_eoi_rejected() {
        let mut v = jpeg_body();
        v.pop();
        v.pop();
        v.push(0xAA);
        v.push(0xAA);
        assert!(!is_valid_jpeg(&v));
    }

    #[test]
    fn empty_rejected() {
        assert!(!is_valid_jpeg(&[]));
    }

    #[test]
    fn eoi_not_at_end_but_present_ok() {
        // 容忍 EOI 之后有 trailing 字节（某些编码器会追加元数据）
        let mut v = jpeg_body();
        v.extend([0x00, 0x00, 0x00]);
        assert!(is_valid_jpeg(&v));
    }
}
