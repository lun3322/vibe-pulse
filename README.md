# Vibe Pulse

写代码时，提醒很容易被塞进另一个窗口。Qoder 还在跑工具，Claude Code 正等权限，终端压在编辑器下面，人已经切去看文档了。

Vibe Pulse 把这些状态做成了一排放在桌面角落的交通灯。绿灯说明会话已经空闲或完成，黄灯亮着时还在工作，红灯则提醒你回来处理输入、权限或失败。它用 Rust 和 windows-rs 编写，直接调用 Win32，没有 WebView，也不会在后台养着一个浏览器内核。

## 它现在能做什么

一台电脑上可以同时挂着多个 Qoder 和 Claude Code 会话。每个会话都有自己的一组灯，Qoder 左侧是绿色短线，Claude Code 左侧是橙色短线，不用点开终端也能分清是谁在忙。

鼠标移到灯上，会看到客户端、来源主机、工作目录、任务摘要、状态和持续时间。提示卡会先找灯的右侧，右边放不下就挪到左侧；多显示器环境下，它会留在灯所在的屏幕里。悬浮窗可以拖动，已经失去联系的会话也可以手动关掉。

Hook 可以从本机 Windows、WSL2 或局域网里的另一台电脑发来。配置窗口不会擅自改 Qoder 或 Claude Code 的文件，只负责生成 JSON 并复制到剪贴板，最后由你决定贴到哪里。

## 为什么它比较轻

Vibe Pulse 会长时间待在后台，所以资源占用从一开始就是个实际问题。

程序发布后是一个原生可执行文件，没有 Electron、WebView、HTML、CSS 和 JavaScript 引擎。界面由 GDI 软件光栅和 Win32 分层窗口直接画出来，Rust 代码也不需要垃圾回收器或额外运行时。

HTTP Hook 服务只开一个轻量后台线程，收到事件以后才解析 JSON、更新会话。没有会话时，交通灯窗口会隐藏；提示卡和配置窗口也只在用到时出现。内存里的会话数据很小，增加会话时，开销按可见会话数量增长。

这些选择让它在 CPU 和内存方面天然比常驻浏览器内核的桌面程序省一些。实际数字会受 Windows 版本、DPI、动画状态和会话数量影响。目前项目还没有一套统一的基准测试，所以这里不写一个看着漂亮却经不起复现的固定数值。

## 状态怎样对应灯光

| 会话状态 | 灯光 |
| --- | --- |
| 空闲或完成 | 绿灯 |
| 工作中 | 黄灯闪烁 |
| 等待输入或权限 | 红灯闪烁 |
| 失败 | 红灯闪烁并保留，直到手动关闭 |

正常结束的会话会先亮绿灯，大约 2 秒后自动消失。失败会话会留下来，免得错误刚出现就被清掉。

## 运行条件

- Windows 10 或 Windows 11
- 构建时需要支持 Rust 2024 Edition 的稳定版 Rust 工具链

## 自己构建

在项目目录运行下面的命令。

```powershell
cargo build --release
```

完成后可以在这里找到程序。

```text
target\release\vibe-pulse.exe
```

## 第一次使用

1. 运行 `vibe-pulse.exe`。
2. 在系统托盘找到交通灯图标，右键选择“配置”。
3. 填写来源主机。这个名字用来区分发送 Hook 的电脑或 WSL 实例，也会出现在悬停提示里。
4. 只在本机使用时，服务地址保持 `127.0.0.1` 即可。
5. 需要接收 WSL2 或局域网设备时，打开“允许 WSL2 / 局域网接入”，从下拉框选择内网 IPv4，然后重启程序。
6. 点击“复制 Qoder 配置”或“复制 Claude Code 配置”。
7. 把剪贴板里的 `hooks` JSON 合并到对应客户端的配置文件。

Vibe Pulse 在固定端口 `17321` 上接收 Hook，请求地址如下。

```text
http://<服务地址>:17321/hooks
```

## 配置放在哪里

程序会在 `vibe-pulse.exe` 同目录创建下面这个文件。

```text
vibe-pulse.settings.json
```

里面只有三项程序设置。

- `token` 保存 HTTP Hook 使用的 Bearer Token
- `allow_lan` 记录是否允许 WSL2 或局域网接入
- `selected_address` 记录复制 Hook 配置时使用的服务地址

首次运行会生成一个 256 位随机 Token，以后打开配置窗口不会重新生成。复制出的 Hook 配置和 HTTP 服务使用同一个 Token。

这个文件不要公开。局域网模式会让程序监听所有本机网络接口，没有正确 Bearer Token 的请求仍会被拒绝。如果准备上传源码，记得检查发布目录，不要把自己运行时生成的 `vibe-pulse.settings.json` 一起传上去。

## Hook 请求长什么样

Vibe Pulse 接收 `POST /hooks` 请求。配置窗口生成的 JSON 会带上这些请求头。

```text
Authorization: Bearer <token>
X-Vibe-Client: qoder | claude-code
X-Vibe-Host: <来源主机>
```

`X-Vibe-Client` 决定灯组使用哪种来源标识。`X-Vibe-Host` 让同一个会话 ID 在不同电脑上仍能分开显示。

程序会处理会话开始、用户提交提示、工具调用、权限请求、通知、停止和会话结束等 Hook。即使客户端漏掉 `SessionStart`，后续事件到达时也会补建灯组。手动关闭某个灯组以后，本次运行期间的后续事件不会把它重新拉回来。

## 代码从哪里看起

如果想改这个项目，可以先从这些文件下手。

- `src/main.rs` 管 Win32 窗口和消息循环
- `src/app.rs` 把 HTTP 事件、会话、绘制和鼠标交互接在一起
- `src/http_server.rs` 接收和校验 HTTP Hook
- `src/model.rs` 维护会话与灯光状态
- `src/drawing.rs` 和 `src/renderer.rs` 绘制透明悬浮窗
- `src/tooltip.rs` 负责提示卡和多显示器边界
- `src/config_window.rs` 负责原生配置窗口

提交修改前，可以跑一遍这些命令。

```powershell
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## 贡献

Issue 和 Pull Request 都欢迎。遇到问题时，把复现步骤、你原本期待看到什么、最后实际出现了什么写清楚，会比一句“不能用”省下很多来回确认的时间。

代码修改尽量只处理眼前的问题。界面改动最好带截图，Hook 或状态逻辑改动最好说明你实际发了什么事件、看到了哪盏灯。

## 许可证

Vibe Pulse 使用 [GNU General Public License v3.0](LICENSE)，SPDX 标识为 `GPL-3.0-only`。你可以使用、研究、修改和分发它；发布修改版时，也需要按 GPL v3 的要求提供对应源码和许可证说明。
