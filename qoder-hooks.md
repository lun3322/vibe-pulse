> ## Documentation Index
> Fetch the complete documentation index at: https://docs.qoder.com/llms.txt
> Use this file to discover all available pages before exploring further.

# Hooks

> 在 Qoder 任务生命周期事件中运行自动化处理程序。

Qoder 使用与 Qoder CLI 兼容的 Hooks。你可以在任务、工具、权限、子智能体、上下文、配置、文件和 Worktree 等事件发生时运行确定性的检查或操作。支持的事件和处理程序类型与 CLI 一致。

<Warning>Hooks 可能访问本地文件或外部服务。使用前请检查命令、服务地址及配置来源，不要运行不可信配置。</Warning>

## 查看当前配置

前往**设置 → 钩子**。页面读取用户级 `~/.qoder/settings.json`，并按事件展示已配置的 Hooks。页面显示的是磁盘配置；配置可见不等于 Hook 已被 Runtime 加载或成功执行，仍需通过实际任务验证。

项目还可使用以下配置文件：

- `${project}/.qoder/settings.json`：项目共享配置。
- `${project}/.qoder/settings.local.json`：仅当前设备使用的项目配置。

不同来源的 Hooks 会合并生效。

## 处理程序类型

| 类型        | 用途                |
| --------- | ----------------- |
| `command` | 运行本地命令或脚本。        |
| `http`    | 向 HTTP 服务发送事件。    |
| `prompt`  | 使用提示词判断是否允许继续。    |
| `agent`   | 交给 Agent 处理并返回结果。 |

## 支持的事件

| 事件                                       | 触发时机             |
| ---------------------------------------- | ---------------- |
| `SessionStart` / `SessionEnd`            | 任务开始 / 任务结束      |
| `UserPromptSubmit`                       | 用户提交指令           |
| `PreToolUse`                             | 工具运行前            |
| `PostToolUse` / `PostToolUseFailure`     | 工具成功 / 失败后       |
| `PermissionRequest` / `PermissionDenied` | 请求权限 / 权限被拒绝     |
| `Stop` / `StopFailure`                   | Agent 停止前 / 停止失败 |
| `SubagentStart` / `SubagentStop`         | 子智能体开始 / 停止      |
| `PreCompact` / `PostCompact`             | 上下文压缩前 / 后       |
| `Notification`                           | 产生通知             |
| `InstructionsLoaded`                     | 加载指令             |
| `ConfigChange`                           | 配置发生变化           |
| `CwdChanged`                             | 当前工作目录变化         |
| `FileChanged`                            | 文件发生变化           |
| `WorktreeCreate` / `WorktreeRemove`      | 创建 / 移除 Worktree |
| `Elicitation` / `ElicitationResult`      | 发起信息请求 / 获得结果    |

## 配置示例

以下示例在工具运行前调用本地脚本：

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "~/.qoder/hooks/check-command.sh"
          }
        ]
      }
    ]
  }
}
```

`matcher` 用于限定触发范围。保存配置后，重新打开**设置 → 钩子**并刷新页面，再运行会触发该事件的低风险任务。检查任务结果以及处理程序自身的日志或输出。

有关匹配规则、输入输出结构和各类处理程序的完整配置，请参见 [Qoder CLI Hooks](/zh/cli/hooks)。
