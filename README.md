# WallpaperRust

双击即换的 Windows 壁纸小工具，零依赖、单文件、纯 Rust 实现。

包含两个独立应用：

| 应用 | 作用 | 权限 |
|------|------|------|
| **DesktopRust.exe** | 双击更换桌面壁纸，完成即退出 | 普通用户 |
| **LockscreenRust.exe** | 预览后确认更换锁屏壁纸 | 管理员（自动提权） |

## 特性

- **零第三方依赖**：手写 Win32 FFI（WinHTTP / SystemParametersInfo / PersonalizationCSP / COM），无任何 crate 依赖
- **单文件 150KB 左右**：DesktopRust ~143KB，LockscreenRust ~151KB，无需安装
- **高质量图源**：Bing 每日壁纸 UHD（4K），每次从近 8 张精选中随机取一张；UHD→HD→Picsum 多级降级
- **下载完整性校验**：JPEG SOI/EOI 标记 + 最小字节三重校验，杜绝半截图片导致黑屏
- **锁屏预览**：设置锁屏前先预览，可"换一张"，满意后再应用（锁屏后无法直接查看效果）
- **多显示器 DPI 感知**：`GetDpiForWindow` + `WM_DPICHANGED`，窗口在不同缩放比的显示器间拖拽时自动调整
- **静默哲学**：桌面版无窗口、无日志、无弹窗，成功即退出

## 使用

### 更换桌面壁纸

双击 `DesktopRust.exe`，壁纸立即更换，程序随即退出。想换就再双击一次。

### 更换锁屏壁纸

1. 双击 `LockscreenRust.exe`，接受 UAC 提权
2. 等待下载（窗口会立即显示"正在下载壁纸..."）
3. 预览图片，不满意点"换一张"，满意点"设为锁屏壁纸"

## 构建

需要 Rust 稳定版工具链（Windows）：

```powershell
cargo build --release
```

产物位于 `target/release/DesktopRust.exe` 与 `target/release/LockscreenRust.exe`。

质量门禁（34 个单元测试）：

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --lib
```

## 工作原理

**桌面壁纸**：下载图片并校验后，先写入持久路径 `%APPDATA%\WallpaperRust\wallpaper.jpg`
（固定文件名覆盖写），再调用 `SystemParametersInfoW(SPI_SETDESKWALLPAPER)`，
传入 `SPIF_UPDATEINIFILE | SPIF_SENDCHANGE` 持久化并广播变更。
壁纸必须指向持久文件——`SystemParametersInfoW` 只登记路径、不复制文件，
若指向临时目录（会被清理或删除），Shell 下次刷新壁纸时读不到文件会回退纯黑。

**锁屏壁纸**：将图片复制到 `%ProgramData%\Microsoft\Windows\Lockscreen.jpg`，
写入 `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\PersonalizationCSP`
（`LockScreenImagePath` 为路径，`LockScreenImageStatus = 1`），
Windows 读取该 CSP 注册表项应用锁屏。因需写 HKLM 与 ProgramData，必须管理员权限。

**图源策略**：Bing API（`HPImageArchive`，n=8）返回多张精选摄影，
零依赖字符串解析提取 `urlbase`，优先下载 `_UHD.jpg`，失败降级 `_1920x1080.jpg`；
所有图源 Fisher-Yates 洗牌后逐个尝试。

**临时文件管理**：下载文件命名 `temp_{时间戳}_{PID}_{序号}.jpg` 避免同秒冲突；
每次下载前清扫 `%TEMP%\WallpaperRust` 中超过 1 小时的崩溃残留文件。

## 关于本项目

WallpaperRust 是对 [WallpaperFlowZero](https://github.com/31i45/WallpaperFlowZero)
（Python 实现）的 Rust 重构。重写动机：

1. **依赖与体积**：原版依赖 aiohttp / asyncio / tkinter，PyInstaller 打包后 14.6 MB；
   Rust 版零运行时依赖，两个 exe 合计约 294 KB
2. **正确性缺陷**：原版下载无完整性校验、`fWinIni` 缺少 `SPIF_SENDCHANGE`、
   随机图源采用有放回抽样（死源会拖累健康源）、锁屏功能为从未生效的死代码
3. **可测试性**：原版核心逻辑与副作用耦合，单元测试数为 0；
   Rust 版按"策略 / 机制 / 编排"分层，纯逻辑策略层全部可单元测试
4. **交互回归本质**：原版常驻后台定时更换，但"换壁纸"本质是一次性动作——
   双击即换、完成即退出才是最短路径；桌面与锁屏权限模型不同，故拆为两个独立应用

## License

[MIT](LICENSE)
