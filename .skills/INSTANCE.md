# INSTANCE.md · WallpaperRust

> 有效期：构建命令、依赖、图源变更时需更新
> 最后验证：2026-10-20（v1.0.3）

## 构建与测试命令
- 构建: `cargo build --release`
- 测试: `cargo test --lib`
- clippy: `cargo clippy --all-targets -- -D warnings`
- fmt: `cargo fmt --all -- --check`

## 数字红线（skill 01 A2/A3, skill 03 A2）

| 数字 | 红线 | 比较符 | 来源 |
|---|---|---|---|
| DesktopRust 产物体积 | ≤ 150 KB | ≤ | release 构建产物 |
| LockscreenRust 产物体积 | ≤ 160 KB | ≤ | release 构建产物 |
| 单元测试数 | ≥ 30 | ≥ | `cargo test --lib` 输出 |
| 直接依赖数 | = 0 | == | Cargo.toml |
| clippy 警告 | = 0 | == | `cargo clippy -D warnings` |
| fmt 差异 | = 0 | == | `cargo fmt --check` |

## 外部输入点（skill 02 B1 实现级 STRIDE）
- Bing JSON API: `https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=8&mkt=zh-CN`
- Picsum 图片: `https://picsum.photos/{w}/{h}`
- 锁屏注册表: `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\PersonalizationCSP`
- ShellExecuteW runas 提权
- 临时文件目录: `%TEMP%\WallpaperRust`

## 核心功能清单（skill 02 B3 故障注入）
1. 下载壁纸（WinHTTP）
2. JPEG 完整性校验（SOI + EOI + 最小字节）
3. 设置桌面壁纸（SystemParametersInfoW SPI_SETDESKWALLPAPER）
4. 设置锁屏壁纸（PersonalizationCSP 注册表）
5. 锁屏预览窗口（OleLoadPicturePath IPicture 渲染）
6. 崩溃残留临时文件清扫（> 1 小时过期）

## 平台接缝机制（skill 01 B3）
- 机制：手写 Win32 extern "system" FFI 声明，集中在 `win32/mod.rs`
- 上层代码（lib.rs / bin / sources / validate / config）零 unsafe 系统调用
- 涉及的系统 DLL：kernel32 / user32 / winhttp / ole32 / oleaut32 / advapi32 / shell32 / gdi32

## 供应链监控（skill 02 B5）
- 监控方式：零第三方依赖，无需 cargo audit / dependabot
- 直接依赖：0 个 crate
- 传递依赖：0 个 crate
- 系统 DLL 风险：由 Windows Update 维护，非 crate 生态

## 违禁词清单（skill 03 A3）
- 词级：（当前无内部术语对外泄漏）
- 概念级：（当前无）

## 环境差异（skill 03 A5）
- 支持 OS：Windows 10 21H2+, Windows 11
- 已知差异：
  - 锁屏 CSP 在 Win10 某些版本可能不生效（设计为"尽力而为"，失败静默退出）
  - DPI 感知需 `SetProcessDPIAware()`，否则多显示器不同缩放时窗口发虚
  - `GetTempPathW` 依赖系统环境变量 TEMP/TMP，极端环境可能返回空
