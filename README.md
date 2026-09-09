# Vibe Pulse

Vibe Pulse 是一个 Windows 原生状态灯，用来显示 Qoder 和 Claude Code 的会话状态。

它会在桌面角落放一组交通灯。绿灯表示空闲或完成，黄灯表示正在工作，红灯表示正在等待输入、权限或出现失败。鼠标移到灯上，可以看到来源主机、工作目录、任务摘要和运行时间。

## 主要功能

- 同时显示多个 Qoder 和 Claude Code 会话
- 支持 Windows、WSL2 和局域网设备发送 HTTP Hook
- 提示卡会根据屏幕空间显示在灯的左侧或右侧
- 悬浮窗可以拖动，也可以手动关闭单个会话
- 托盘配置窗口可以直接复制 Qoder 和 Claude Code 配置
- 首次运行自动生成随机 Bearer Token

## 资源占用

程序使用 Rust、windows-rs 和原生 Win32 编写，不包含 Electron、WebView 或浏览器内核。界面直接用 GDI 和分层窗口绘制，HTTP 服务只在收到事件时处理数据，适合长时间放在后台运行。

实际 CPU 和内存占用会随 DPI、动画状态和会话数量变化，因此项目暂不提供未经统一测试的固定数值。

## 构建

需要 Windows 10 或 Windows 11，以及支持 Rust 2024 Edition 的稳定版 Rust 工具链。

```powershell
cargo build --release
```

生成文件位于 `target\release\vibe-pulse.exe`。

## 使用

1. 运行 `vibe-pulse.exe`。
2. 右键托盘中的交通灯图标，打开“配置”。
3. 本机使用时保留 `127.0.0.1`。
4. WSL2 或局域网使用时，打开局域网接入并选择内网 IPv4，然后重启程序。
5. 复制 Qoder 或 Claude Code 配置，再合并到对应客户端的配置文件。

Hook 服务监听 `17321` 端口。

```text
http://<服务地址>:17321/hooks
```

## 状态说明

| 状态 | 显示 |
| --- | --- |
| 空闲或完成 | 绿灯 |
| 工作中 | 黄灯闪烁 |
| 等待输入或权限 | 红灯闪烁 |
| 失败 | 红灯闪烁并保留 |

正常结束的会话会在约 2 秒后消失。失败会话会一直保留，直到手动关闭。

## 配置与安全

程序会在 EXE 同目录创建 `vibe-pulse.settings.json`，其中保存服务地址、局域网开关和 Bearer Token。

不要公开这个文件。开启局域网接入后，程序会监听本机网络接口，没有正确 Token 的请求会被拒绝。

## 开发检查

```powershell
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## 许可证

项目使用 [GNU General Public License v3.0](LICENSE)，SPDX 标识为 `GPL-3.0-only`。
